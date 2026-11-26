pub mod error;
pub mod logging;
pub mod modules;
pub mod notifications;

use modules::{app, settings};
use notifications::NotificationSettings;

/// Point d'entrée de l'application, appelé par `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::log_panics();
    tauri::Builder::default()
        .plugin(logging::plugin())
        .plugin(tauri_plugin_notification::init())
        .manage(NotificationSettings::default())
        .invoke_handler(tauri::generate_handler![
            app::app_version,
            notifications::notifications_send,
            notifications::notifications_is_enabled,
            notifications::notifications_set_enabled,
            settings::settings_get,
            settings::settings_save,
        ])
        .setup(|app| {
            log::info!("démarrage de la version {}", env!("CARGO_PKG_VERSION"));
            settings::init(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("impossible de démarrer l'application");
}
