pub mod error;
pub mod modules;
pub mod notifications;

use modules::app;
use notifications::NotificationSettings;

/// Point d'entrée de l'application, appelé par `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(NotificationSettings::default())
        .invoke_handler(tauri::generate_handler![
            app::app_version,
            notifications::notifications_send,
            notifications::notifications_is_enabled,
            notifications::notifications_set_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("impossible de démarrer l'application");
}
