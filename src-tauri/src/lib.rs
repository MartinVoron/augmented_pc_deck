pub mod error;
pub mod logging;
pub mod modules;
pub mod notifications;
pub mod tokens;

use modules::app;
use notifications::NotificationSettings;
use tokens::TokenStore;

/// Point d'entrée de l'application, appelé par `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::log_panics();
    tauri::Builder::default()
        .plugin(logging::plugin())
        .plugin(tauri_plugin_notification::init())
        .manage(NotificationSettings::default())
        .manage(TokenStore::system())
        .invoke_handler(tauri::generate_handler![
            app::app_version,
            notifications::notifications_send,
            notifications::notifications_is_enabled,
            notifications::notifications_set_enabled,
            tokens::tokens_status,
            tokens::tokens_set,
            tokens::tokens_delete,
        ])
        .setup(|_app| {
            log::info!("démarrage de la version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("impossible de démarrer l'application");
}
