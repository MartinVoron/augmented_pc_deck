//! Musique : morceau en cours et contrôle de la lecture.
//!
//! Sous Windows, les informations viennent des sessions média (`GlobalSystemMediaTransport
//! ControlsSessionManager`) : la même source que le panneau média de Windows, donc ça
//! marche avec Spotify, les navigateurs, le Lecteur multimédia, VLC…
//! Sous Linux, elles viennent des lecteurs MPRIS (D-Bus), la même source que le
//! panneau média de GNOME ou de KDE.
//! L'interface interroge [`music_now_playing`] toutes les secondes.

mod platform;

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};

/// État de lecture de la session en cours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

/// Ce qui joue en ce moment.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Application qui joue (« Spotify », « Chrome »…).
    pub source: String,
    pub status: PlaybackStatus,
    /// Position au moment `updated_at`, en millisecondes.
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    /// Heure de la dernière mise à jour de la position, en millisecondes depuis 1970 :
    /// l'interface fait avancer la barre à partir de là.
    pub updated_at: u64,
    /// Pochette en `data:` URL.
    pub thumbnail: Option<String>,
    pub can_play_pause: bool,
    pub can_next: bool,
    pub can_previous: bool,
}

/// Clé du morceau et sa pochette.
type CachedThumbnail = Option<(String, Option<String>)>;

/// Pochette du dernier morceau lu, pour ne pas la relire à chaque seconde.
#[derive(Default, Clone)]
pub struct MusicState {
    #[cfg_attr(not(any(windows, target_os = "linux")), allow(dead_code))]
    thumbnail: Arc<Mutex<CachedThumbnail>>,
}

#[cfg_attr(not(any(windows, target_os = "linux")), allow(dead_code))]
impl MusicState {
    /// Renvoie la pochette gardée pour `key`, ou la lit avec `read` et la garde.
    fn thumbnail(&self, key: &str, read: impl FnOnce() -> Option<String>) -> Option<String> {
        let Ok(mut cache) = self.thumbnail.lock() else {
            return read();
        };
        if let Some((cached_key, thumbnail)) = cache.as_ref() {
            if cached_key == key {
                return thumbnail.clone();
            }
        }
        let thumbnail = read();
        *cache = Some((key.to_string(), thumbnail.clone()));
        thumbnail
    }
}

/// Nom lisible de l'application qui joue, à partir de son identifiant Windows.
pub fn source_name(app_id: &str) -> String {
    let lower = app_id.to_ascii_lowercase();
    let known = [
        ("spotify", "Spotify"),
        ("zunemusic", "Lecteur multimédia"),
        ("chrome", "Chrome"),
        ("msedge", "Edge"),
        ("firefox", "Firefox"),
        ("opera", "Opera"),
        ("brave", "Brave"),
        ("vlc", "VLC"),
        ("deezer", "Deezer"),
        ("applemusic", "Apple Music"),
        ("itunes", "iTunes"),
    ];
    if let Some((_, name)) = known.iter().find(|(key, _)| lower.contains(key)) {
        return (*name).to_string();
    }
    let name = app_id.split(['!', '\\', '/']).next_back().unwrap_or(app_id);
    let name = name.split('!').next().unwrap_or(name);
    name.strip_suffix(".exe")
        .or_else(|| name.strip_suffix(".EXE"))
        .unwrap_or(name)
        .to_string()
}

/// Convertit une date Windows (intervalles de 100 ns depuis 1601) en millisecondes depuis 1970.
pub fn windows_time_to_unix_ms(ticks: i64) -> Option<u64> {
    const EPOCH_DIFFERENCE: i64 = 116_444_736_000_000_000;
    let unix = ticks.checked_sub(EPOCH_DIFFERENCE)?;
    u64::try_from(unix / 10_000).ok().filter(|&ms| ms > 0)
}

/// Durée Windows (intervalles de 100 ns) en millisecondes.
pub fn timespan_to_ms(duration: i64) -> u64 {
    u64::try_from(duration / 10_000).unwrap_or(0)
}

/// Type d'une image d'après ses premiers octets (les lecteurs MPRIS ne le donnent pas).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG") {
        Some("image/png")
    } else if bytes.starts_with(b"\xFF\xD8\xFF") {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF8") {
        Some("image/gif")
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Commande de lecture envoyée à l'application qui joue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MusicAction {
    PlayPause,
    Next,
    Previous,
}

/// Renvoie le morceau en cours, ou `None` quand rien ne joue.
#[tauri::command]
pub async fn music_now_playing(state: State<'_, MusicState>) -> AppResult<Option<NowPlaying>> {
    // Les appels au système attendent leur réponse : on les fait hors du fil principal.
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || platform::now_playing(&state))
        .await
        .map_err(|e| AppError::Other(format!("lecture de la musique interrompue : {e}")))?
}

/// Envoie une commande (lecture/pause, suivant, précédent) à l'application qui joue.
#[tauri::command]
pub async fn music_control(action: MusicAction) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || platform::control(action))
        .await
        .map_err(|e| AppError::Other(format!("commande de lecture interrompue : {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gives_readable_source_names() {
        assert_eq!(source_name("Spotify.exe"), "Spotify");
        assert_eq!(
            source_name("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic"),
            "Lecteur multimédia"
        );
        assert_eq!(source_name("MSEdge"), "Edge");
        assert_eq!(source_name(r"C:\Outils\Lecteur.exe"), "Lecteur");
        assert_eq!(source_name("Foobar2000.exe"), "Foobar2000");
    }

    #[test]
    fn converts_windows_times() {
        // 1er janvier 2024 à 00:00 UTC.
        assert_eq!(
            windows_time_to_unix_ms(133_485_408_000_000_000),
            Some(1_704_067_200_000)
        );
        assert_eq!(windows_time_to_unix_ms(0), None);
        assert_eq!(timespan_to_ms(2_150_000_000), 215_000);
        assert_eq!(timespan_to_ms(-5), 0);
    }

    #[test]
    fn recognises_cover_images() {
        assert_eq!(image_mime(b"\x89PNG\r\n\x1a\n"), Some("image/png"));
        assert_eq!(image_mime(b"\xFF\xD8\xFF\xE0"), Some("image/jpeg"));
        assert_eq!(image_mime(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
        assert_eq!(image_mime(b"<svg"), None);
        assert_eq!(image_mime(b""), None);
    }

    #[test]
    fn reads_actions_from_the_interface() {
        let action: MusicAction = serde_json::from_str(r#""playPause""#).unwrap();
        assert_eq!(action, MusicAction::PlayPause);
        assert!(serde_json::from_str::<MusicAction>(r#""stop""#).is_err());
    }

    #[test]
    fn keeps_the_thumbnail_of_the_same_track() {
        let state = MusicState::default();
        let mut reads = 0;
        let mut read = |value: &str| {
            reads += 1;
            Some(value.to_string())
        };
        assert_eq!(state.thumbnail("a", || read("1")), Some("1".into()));
        assert_eq!(state.thumbnail("a", || read("2")), Some("1".into()));
        assert_eq!(state.thumbnail("b", || read("3")), Some("3".into()));
        assert_eq!(reads, 2);
    }
}
