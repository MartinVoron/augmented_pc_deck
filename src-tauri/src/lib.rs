pub mod error;
pub mod modules;

use modules::app;

/// Point d'entrée de l'application, appelé par `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app::app_version])
        .run(tauri::generate_context!())
        .expect("impossible de démarrer l'application");
}
