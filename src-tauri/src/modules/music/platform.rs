//! Accès aux lecteurs de musique : sessions média de Windows, MPRIS sous Linux.
//! Ailleurs, rien ne joue.

#[cfg(target_os = "linux")]
pub use linux_impl::{control, now_playing};
#[cfg(windows)]
pub use windows_impl::{control, now_playing};

#[cfg(not(any(windows, target_os = "linux")))]
pub fn now_playing(
    _state: &super::MusicState,
) -> crate::error::AppResult<Option<super::NowPlaying>> {
    Ok(None)
}

#[cfg(not(any(windows, target_os = "linux")))]
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

#[cfg(target_os = "linux")]
mod linux_impl {
    //! Lecteurs MPRIS (D-Bus) : Spotify, Firefox, Chrome, VLC, Rhythmbox…

    use std::time::Duration;

    use base64::Engine;
    use mpris::{PlaybackStatus as Status, Player, PlayerFinder};

    use super::super::{image_mime, MusicAction, MusicState, NowPlaying, PlaybackStatus};
    use crate::error::{AppError, AppResult};

    /// Taille maximale d'une pochette lue.
    const MAX_THUMBNAIL_BYTES: u64 = 2 * 1024 * 1024;

    /// Délai maximal d'une réponse d'un lecteur, pour ne pas bloquer la tuile.
    const DBUS_TIMEOUT_MS: i32 = 500;

    fn mpris_error(e: impl std::fmt::Display) -> AppError {
        AppError::Other(format!("les lecteurs de musique ne répondent pas : {e}"))
    }

    /// Ordre de préférence : un lecteur qui joue, puis en pause, puis arrêté.
    fn rank(status: Status) -> u8 {
        match status {
            Status::Playing => 0,
            Status::Paused => 1,
            Status::Stopped => 2,
        }
    }

    /// Lecteur à afficher, ou `None` si aucun lecteur n'est ouvert.
    fn current_player() -> AppResult<Option<Player>> {
        let finder = PlayerFinder::new().map_err(mpris_error)?;
        let players = match finder.find_all() {
            Ok(players) => players,
            Err(mpris::FindingError::NoPlayerFound) => return Ok(None),
            Err(e) => return Err(mpris_error(e)),
        };
        Ok(players
            .into_iter()
            .map(|mut player| {
                player.set_dbus_timeout_ms(DBUS_TIMEOUT_MS);
                let status = player.get_playback_status().unwrap_or(Status::Stopped);
                (rank(status), player)
            })
            .min_by_key(|(rank, _)| *rank)
            .map(|(_, player)| player))
    }

    pub fn now_playing(state: &MusicState) -> AppResult<Option<NowPlaying>> {
        let Some(player) = current_player()? else {
            return Ok(None);
        };
        let metadata = player.get_metadata().map_err(mpris_error)?;
        let title = metadata.title().unwrap_or_default().to_string();
        if title.is_empty() {
            return Ok(None);
        }
        let artist = metadata.artists().unwrap_or_default().join(", ");
        let album = metadata.album_name().unwrap_or_default().to_string();
        let source = player.identity().to_string();
        let status = match player.get_playback_status().map_err(mpris_error)? {
            Status::Playing => PlaybackStatus::Playing,
            Status::Paused => PlaybackStatus::Paused,
            Status::Stopped => PlaybackStatus::Stopped,
        };
        let duration = metadata.length().map_or(0, |d| d.as_millis() as u64);
        let position = player
            .get_position()
            .map_or(0, |d: Duration| d.as_millis() as u64);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as u64);

        let key = format!("{source}|{title}|{artist}|{album}");
        let art_url = metadata.art_url().map(str::to_string);
        let thumbnail = state.thumbnail(&key, || art_url.as_deref().and_then(read_thumbnail));

        let can_control = player.can_control().unwrap_or(false);
        Ok(Some(NowPlaying {
            title,
            artist,
            album,
            source,
            status,
            position_ms: (duration > 0).then_some(position.min(duration)),
            duration_ms: (duration > 0).then_some(duration),
            updated_at: now,
            thumbnail,
            can_play_pause: can_control
                && (player.can_pause().unwrap_or(false) || player.can_play().unwrap_or(false)),
            can_next: can_control && player.can_go_next().unwrap_or(false),
            can_previous: can_control && player.can_go_previous().unwrap_or(false),
        }))
    }

    /// Envoie une commande de lecture au lecteur affiché.
    pub fn control(action: MusicAction) -> AppResult<()> {
        let player =
            current_player()?.ok_or_else(|| AppError::NotFound("La musique en cours".into()))?;
        match action {
            MusicAction::PlayPause => player.play_pause(),
            MusicAction::Next => player.next(),
            MusicAction::Previous => player.previous(),
        }
        .map_err(mpris_error)
    }

    /// Lit la pochette (fichier local ou adresse web) et la renvoie en `data:` URL.
    fn read_thumbnail(art_url: &str) -> Option<String> {
        let url = url::Url::parse(art_url).ok()?;
        let bytes = match url.scheme() {
            "file" => {
                use std::io::Read;
                let path = url.to_file_path().ok()?;
                let mut bytes = Vec::new();
                std::fs::File::open(path)
                    .ok()?
                    .take(MAX_THUMBNAIL_BYTES)
                    .read_to_end(&mut bytes)
                    .ok()?;
                bytes
            }
            "http" | "https" => tauri::async_runtime::block_on(async {
                let response = crate::http::client().get(url).send().await.ok()?;
                if !response.status().is_success()
                    || response.content_length().unwrap_or(0) > MAX_THUMBNAIL_BYTES
                {
                    return None;
                }
                let bytes = response.bytes().await.ok()?;
                (bytes.len() as u64 <= MAX_THUMBNAIL_BYTES).then(|| bytes.to_vec())
            })?,
            _ => return None,
        };
        let mime = image_mime(&bytes)?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        Some(format!("data:{mime};base64,{encoded}"))
    }
}
