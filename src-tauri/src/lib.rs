pub mod display;
pub mod dock;
pub mod error;
pub mod http;
#[cfg(target_os = "linux")]
pub mod layer;
pub mod logging;
pub mod modules;
pub mod notifications;
pub mod tokens;
pub mod window;

use modules::claude::{self, ClaudeState};
use modules::gmail::{self, GmailState};
use modules::music::{self, MusicState};
use modules::processes::{self, ProcessMonitor};
use modules::timers::{self, Timers};
use modules::{app, github, launcher, settings, weather};
use notifications::NotificationSettings;
use tokens::TokenStore;

/// Point d'entrée de l'application, appelé par `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::log_panics();
    // Avant GTK : en session Wayland, choisir comment l'encoche se colle au bord.
    let backend = display::setup();
    tauri::Builder::default()
        .plugin(logging::plugin())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(window::state_plugin())
        .plugin(window::autostart_plugin())
        .manage(dock::DockState::default())
        .manage(NotificationSettings::default())
        .manage(TokenStore::system())
        .manage(ProcessMonitor::default())
        .manage(Timers::default())
        .manage(MusicState::default())
        .manage(GmailState::default())
        .manage(ClaudeState::default())
        .on_window_event(window::on_window_event)
        .invoke_handler(tauri::generate_handler![
            app::app_version,
            notifications::notifications_send,
            notifications::notifications_is_enabled,
            notifications::notifications_set_enabled,
            settings::settings_get,
            settings::settings_save,
            window::window_autostart_get,
            dock::dock_status,
            dock::dock_set_panel,
            dock::dock_set_notch,
            dock::dock_pointer,
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
            timers::timers_list,
            timers::timers_start,
            timers::timers_pause,
            timers::timers_reset,
            music::music_now_playing,
            music::music_control,
            github::github_connect,
            github::github_account,
            github::github_disconnect,
            github::github_activity,
            gmail::gmail_connect,
            gmail::gmail_account,
            gmail::gmail_disconnect,
            gmail::gmail_unread,
            claude::claude_status,
            claude::claude_install_hook,
            claude::claude_regenerate_token,
            claude::claude_events,
            claude::claude_clear,
            weather::weather_forecast,
            weather::weather_search,
        ])
        .setup(move |app| {
            log::info!("démarrage de la version {}", env!("CARGO_PKG_VERSION"));
            if !display::summary().is_empty() {
                log::info!("affichage : {}", display::summary());
            }
            match backend {
                display::Backend::Wayland => {
                    log::info!("session Wayland : encoche par le layer shell");
                }
                display::Backend::Xwayland => log::info!(
                    "session Wayland : affichage par XWayland (GDK_BACKEND=x11) pour l'encoche"
                ),
                display::Backend::Default => {}
            }
            settings::init(app.handle())?;
            // La fenêtre est créée masquée (tauri.conf.json) : sous Wayland, l'encoche
            // doit se coller au bord avant d'être affichée.
            if !dock::shows_itself(app.handle()) {
                window::show(app.handle());
            }
            window::create_tray(app.handle())?;
            claude::start(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("impossible de démarrer l'application")
        .run(|app, event| {
            // Fermer la seule fenêtre quitte l'application, sauf pendant son
            // remplacement (encoche sous Wayland). « Quitter » donne un code et
            // n'est pas bloqué.
            if let tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                if dock::recreating(app) {
                    api.prevent_exit();
                }
            }
        });
}
