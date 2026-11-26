pub mod error;
pub mod logging;
pub mod modules;
pub mod notifications;
pub mod tokens;
pub mod window;

use modules::processes::{self, ProcessMonitor};
use modules::{app, launcher, settings};
use notifications::NotificationSettings;
use tokens::TokenStore;

/// Point d'entrée de l'application, appelé par `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::log_panics();
    tauri::Builder::default()
        .plugin(logging::plugin())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(window::state_plugin())
        .plugin(window::autostart_plugin())
        .manage(NotificationSettings::default())
        .manage(TokenStore::system())
        .manage(ProcessMonitor::default())
        .invoke_handler(tauri::generate_handler![
            app::app_version,
            notifications::notifications_send,
            notifications::notifications_is_enabled,
            notifications::notifications_set_enabled,
            settings::settings_get,
            settings::settings_save,
            window::window_autostart_get,
            window::window_autostart_set,
            tokens::tokens_status,
            tokens::tokens_set,
            tokens::tokens_delete,
            launcher::launcher_launch,
            launcher::launcher_icon,
            launcher::launcher_read_image,
            processes::processes_usage,
            processes::processes_list,
            processes::processes_kill,
        ])
        .setup(|app| {
            log::info!("démarrage de la version {}", env!("CARGO_PKG_VERSION"));
            settings::init(app.handle())?;
            window::create_tray(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("impossible de démarrer l'application");
}
