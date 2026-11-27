//! Accès aux sessions média de Windows. Hors de Windows, rien ne joue.

#[cfg(windows)]
pub use windows_impl::{control, now_playing};

#[cfg(not(windows))]
pub fn now_playing(
    _state: &super::MusicState,
) -> crate::error::AppResult<Option<super::NowPlaying>> {
    Ok(None)
}

#[cfg(not(windows))]
pub fn control(_action: super::MusicAction) -> crate::error::AppResult<()> {
    Err(crate::error::AppError::NotFound(
        "La musique en cours".into(),
    ))
}

#[cfg(windows)]
mod windows_impl {
    use base64::Engine;
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as SessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };
    use windows::Storage::Streams::{
        Buffer, DataReader, IRandomAccessStreamReference, InputStreamOptions,
    };
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

    use super::super::{
        source_name, timespan_to_ms, windows_time_to_unix_ms, MusicAction, MusicState, NowPlaying,
        PlaybackStatus,
    };
    use crate::error::{AppError, AppResult};

    /// Taille maximale d'une pochette lue (les pochettes font quelques dizaines de Ko).
    const MAX_THUMBNAIL_BYTES: u64 = 2 * 1024 * 1024;

    fn windows_error(e: windows::core::Error) -> AppError {
        AppError::Other(format!(
            "les sessions média de Windows ne répondent pas : {e}"
        ))
    }

    /// Exécute `f` dans un fil initialisé pour les appels WinRT.
    pub(super) fn with_winrt<T>(f: impl FnOnce() -> T) -> T {
        // SAFETY : chaque initialisation réussie est suivie de sa libération.
        let init = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let result = f();
        if init.is_ok() {
            unsafe { CoUninitialize() };
        }
        result
    }

    /// Session média active, ou `None` si aucune application ne joue.
    pub(super) fn current_session() -> AppResult<Option<Session>> {
        let manager = SessionManager::RequestAsync()
            .and_then(|op| op.join())
            .map_err(windows_error)?;
        Ok(manager.GetCurrentSession().ok())
    }

    pub fn now_playing(state: &MusicState) -> AppResult<Option<NowPlaying>> {
        with_winrt(|| {
            let Some(session) = current_session()? else {
                return Ok(None);
            };
            read_session(&session, state).map_err(windows_error)
        })
    }

    /// Envoie une commande de lecture à la session active.
    pub fn control(action: MusicAction) -> AppResult<()> {
        with_winrt(|| {
            let session = current_session()?
                .ok_or_else(|| AppError::NotFound("La musique en cours".into()))?;
            let operation = match action {
                MusicAction::PlayPause => session.TryTogglePlayPauseAsync(),
                MusicAction::Next => session.TrySkipNextAsync(),
                MusicAction::Previous => session.TrySkipPreviousAsync(),
            };
            let accepted = operation.and_then(|op| op.join()).map_err(windows_error)?;
            if accepted {
                Ok(())
            } else {
                Err(AppError::PermissionDenied(
                    "l'application qui joue refuse cette commande".into(),
                ))
            }
        })
    }

    fn read_session(
        session: &Session,
        state: &MusicState,
    ) -> windows::core::Result<Option<NowPlaying>> {
        let properties = session.TryGetMediaPropertiesAsync()?.join()?;
        let title = properties.Title()?.to_string();
        if title.is_empty() {
            return Ok(None);
        }
        let artist = properties.Artist()?.to_string();
        let album = properties.AlbumTitle()?.to_string();
        let source = source_name(&session.SourceAppUserModelId()?.to_string());

        let playback = session.GetPlaybackInfo()?;
        let status = match playback.PlaybackStatus()? {
            Status::Playing => PlaybackStatus::Playing,
            Status::Paused => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped,
        };
        let controls = playback.Controls()?;

        let timeline = session.GetTimelineProperties()?;
        let duration = timespan_to_ms(timeline.EndTime()?.Duration)
            .saturating_sub(timespan_to_ms(timeline.StartTime()?.Duration));
        let position = timespan_to_ms(timeline.Position()?.Duration);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as u64);
        let updated_at = windows_time_to_unix_ms(timeline.LastUpdatedTime()?.UniversalTime)
            .filter(|&t| t <= now)
            .unwrap_or(now);

        let key = format!("{source}|{title}|{artist}|{album}");
        let thumbnail = state.thumbnail(&key, || {
            properties
                .Thumbnail()
                .ok()
                .and_then(|reference| read_thumbnail(&reference).ok())
        });

        Ok(Some(NowPlaying {
            title,
            artist,
            album,
            source,
            status,
            position_ms: (duration > 0).then_some(position.min(duration)),
            duration_ms: (duration > 0).then_some(duration),
            updated_at,
            thumbnail,
            can_play_pause: controls.IsPlayPauseToggleEnabled()?,
            can_next: controls.IsNextEnabled()?,
            can_previous: controls.IsPreviousEnabled()?,
        }))
    }

    /// Lit la pochette et la renvoie en `data:` URL.
    fn read_thumbnail(reference: &IRandomAccessStreamReference) -> windows::core::Result<String> {
        let stream = reference.OpenReadAsync()?.join()?;
        let size = stream.Size()?.min(MAX_THUMBNAIL_BYTES) as u32;
        let buffer = Buffer::Create(size)?;
        let buffer = stream
            .ReadAsync(&buffer, size, InputStreamOptions::None)?
            .join()?;
        let reader = DataReader::FromBuffer(&buffer)?;
        let mut bytes = vec![0u8; buffer.Length()? as usize];
        reader.ReadBytes(&mut bytes)?;
        let mime = stream.ContentType()?.to_string();
        let mime = if mime.starts_with("image/") {
            mime
        } else {
            "image/png".into()
        };
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        Ok(format!("data:{mime};base64,{encoded}"))
    }
}
