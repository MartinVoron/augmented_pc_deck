//! Claude : réception des notifications de Claude Code.
//!
//! Claude Code lance un « hook » (une commande) quand il attend une réponse ou
//! qu'il a fini. Ce hook est un petit script PowerShell, créé par l'application,
//! qui transmet l'événement au serveur local (voir [`server`]). Les événements sont
//! gardés en mémoire, envoyés à l'interface avec [`CHANGED_EVENT`], et signalés par
//! une notification Windows.
//!
//! Mise en place : voir `docs/claude-hook.md`.

mod events;
mod hook;
mod server;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::error::{AppError, AppResult};
use crate::notifications;
use crate::tokens::{TokenKind, TokenStore};

pub use events::ClaudeEvent;

/// Événement envoyé à l'interface à chaque changement, avec tous les événements.
pub const CHANGED_EVENT: &str = "claude-events-changed";

/// État du serveur local.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum ServerState {
    Starting,
    Listening,
    /// Le port est pris (une autre instance de l'application ?) ou refusé.
    Failed {
        error: String,
    },
}

/// Mémoire du module.
pub struct ClaudeState {
    events: Mutex<VecDeque<ClaudeEvent>>,
    next_id: AtomicU64,
    server: Mutex<ServerState>,
}

impl Default for ClaudeState {
    fn default() -> Self {
        Self {
            events: Mutex::default(),
            next_id: AtomicU64::new(1),
            server: Mutex::new(ServerState::Starting),
        }
    }
}

impl ClaudeState {
    fn events(&self) -> Vec<ClaudeEvent> {
        self.events
            .lock()
            .map(|e| e.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn set_server(&self, state: ServerState) {
        if let Ok(mut server) = self.server.lock() {
            *server = state;
        }
    }
}

/// État de la mise en place, affiché dans les paramètres.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeStatus {
    pub server: ServerState,
    pub port: u16,
    /// Emplacement du script du hook.
    pub script_path: String,
    pub script_installed: bool,
    /// Bloc à ajouter dans `~/.claude/settings.json`.
    pub settings_snippet: String,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// Nouveau jeton aléatoire (256 bits), utilisable tel quel dans un script.
fn new_token() -> AppResult<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| AppError::Other(format!("aléa indisponible : {e}")))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

/// Jeton du serveur local ; il est créé au premier lancement.
fn ensure_token(store: &TokenStore) -> AppResult<String> {
    if let Some(token) = store.get(TokenKind::Claude)? {
        return Ok(token.expose().to_string());
    }
    let token = new_token()?;
    store.set(TokenKind::Claude, &token)?;
    Ok(token)
}

fn script_path<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Other(format!("dossier de l'application introuvable : {e}")))?;
    Ok(dir.join(hook::SCRIPT_NAME))
}

fn status<R: Runtime>(app: &AppHandle<R>, state: &ClaudeState) -> AppResult<ClaudeStatus> {
    let path = script_path(app)?;
    Ok(ClaudeStatus {
        server: state
            .server
            .lock()
            .map(|s| s.clone())
            .unwrap_or(ServerState::Starting),
        port: server::PORT,
        script_installed: path.exists(),
        settings_snippet: hook::settings_snippet(&path),
        script_path: path.display().to_string(),
    })
}

/// Reçoit le corps d'une requête du hook.
fn receive<R: Runtime>(app: &AppHandle<R>, body: &[u8]) -> server::Status {
    let state = app.state::<ClaudeState>();
    let id = state.next_id.fetch_add(1, Ordering::Relaxed);
    let event = match events::parse(body, id, now_ms()) {
        Ok(Some(event)) => event,
        Ok(None) => return server::Status::Accepted,
        Err(e) => {
            log::warn!("événement Claude illisible : {e}");
            return server::Status::BadRequest;
        }
    };
    log::info!(
        "événement Claude reçu : {:?} ({})",
        event.kind,
        event.project
    );
    let notification = events::notification(&event);
    if let Ok(mut list) = state.events.lock() {
        events::push(&mut list, event);
    }
    emit_changed(app, &state);
    if let Err(e) = notifications::notify(app, &notification) {
        log::warn!("notification Claude impossible : {e}");
    }
    server::Status::Accepted
}

/// Envoie tous les événements à l'interface.
fn emit_changed<R: Runtime>(app: &AppHandle<R>, state: &ClaudeState) {
    if let Err(e) = app.emit(CHANGED_EVENT, state.events()) {
        log::warn!("impossible de prévenir l'interface d'un événement Claude : {e}");
    }
}

/// Démarre le serveur local. Appelé une fois, au lancement de l'application.
pub fn start<R: Runtime>(app: &AppHandle<R>) {
    if let Err(e) = ensure_token(&app.state::<TokenStore>()) {
        log::warn!("jeton du serveur Claude indisponible : {e}");
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<ClaudeState>();
        let listener = match server::bind(server::PORT).await {
            Ok(listener) => listener,
            Err(e) => {
                let error = format!("le port {} est indisponible : {e}", server::PORT);
                log::warn!("serveur Claude : {error}");
                state.set_server(ServerState::Failed { error });
                return;
            }
        };
        log::info!("serveur Claude à l'écoute sur 127.0.0.1:{}", server::PORT);
        state.set_server(ServerState::Listening);
        let token = || {
            app.state::<TokenStore>()
                .get(TokenKind::Claude)
                .ok()
                .flatten()
                .map(|t| t.expose().to_string())
        };
        server::run(listener, token, |body| receive(&app, body)).await;
    });
}

/// État du serveur et du script, pour les paramètres.
#[tauri::command]
pub fn claude_status<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, ClaudeState>,
) -> AppResult<ClaudeStatus> {
    status(&app, &state)
}

/// Crée (ou recrée) le script du hook, avec le jeton actuel.
#[tauri::command]
pub fn claude_install_hook<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, ClaudeState>,
    store: State<'_, TokenStore>,
) -> AppResult<ClaudeStatus> {
    let token = ensure_token(&store)?;
    let path = script_path(&app)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, hook::script(&token, server::PORT))?;
    log::info!("script du hook Claude écrit dans {}", path.display());
    status(&app, &state)
}

/// Remplace le jeton (par exemple si le script a été copié ailleurs).
/// Le script est réécrit s'il existe ; l'ancien jeton ne marche plus.
#[tauri::command]
pub fn claude_regenerate_token<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, ClaudeState>,
    store: State<'_, TokenStore>,
) -> AppResult<ClaudeStatus> {
    store.set(TokenKind::Claude, &new_token()?)?;
    if script_path(&app)?.exists() {
        return claude_install_hook(app, state, store);
    }
    status(&app, &state)
}

/// Événements reçus depuis le lancement, du plus récent au plus ancien.
#[tauri::command]
pub fn claude_events(state: State<'_, ClaudeState>) -> Vec<ClaudeEvent> {
    state.events()
}

/// Efface tous les événements.
#[tauri::command]
pub fn claude_clear<R: Runtime>(app: AppHandle<R>, state: State<'_, ClaudeState>) {
    if let Ok(mut list) = state.events.lock() {
        list.clear();
    }
    emit_changed(&app, &state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::MemoryBackend;

    #[test]
    fn the_token_is_created_once_then_kept() {
        let store = TokenStore::new(MemoryBackend::default());
        let first = ensure_token(&store).unwrap();
        assert_eq!(first.len(), 43);
        assert_eq!(ensure_token(&store).unwrap(), first);
    }

    #[test]
    fn server_state_is_tagged_for_the_interface() {
        let json = serde_json::to_value(ServerState::Failed {
            error: "port pris".into(),
        })
        .unwrap();
        assert_eq!(json["state"], "failed");
        assert_eq!(json["error"], "port pris");
    }
}
