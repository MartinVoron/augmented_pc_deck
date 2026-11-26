//! Paramètres : configuration enregistrée sur le PC.
//!
//! Le fichier `config.json` est dans `%APPDATA%\fr.martinvoron.augmented-pc-deck\`.
//! Il porte un numéro de version (voir [`config::CURRENT_VERSION`]) pour pouvoir
//! migrer les anciens fichiers quand le format change.

pub mod config;
pub mod store;

use tauri::{AppHandle, Manager, Runtime, State};

pub use config::Config;
pub use store::ConfigStore;

use crate::error::{AppError, AppResult};
use crate::notifications::NotificationSettings;

/// Charge la configuration au démarrage et la rend disponible aux commandes.
pub fn init<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::Other(format!("dossier de configuration introuvable : {e}")))?;
    let store = ConfigStore::load(&dir);
    log::info!("configuration chargée depuis {}", store.path().display());
    apply(app, &store.get());
    app.manage(store);
    Ok(())
}

/// Applique les réglages qui concernent d'autres parties de l'application.
fn apply<R: Runtime>(app: &AppHandle<R>, config: &Config) {
    app.state::<NotificationSettings>()
        .set_enabled(config.general.notifications_enabled);
}

/// Renvoie la configuration actuelle.
#[tauri::command]
pub fn settings_get(store: State<'_, ConfigStore>) -> AppResult<Config> {
    Ok(store.get())
}

/// Enregistre une nouvelle configuration et l'applique tout de suite.
#[tauri::command]
pub fn settings_save<R: Runtime>(
    app: AppHandle<R>,
    store: State<'_, ConfigStore>,
    config: Config,
) -> AppResult<Config> {
    let saved = store.set(config)?;
    apply(&app, &saved);
    Ok(saved)
}
