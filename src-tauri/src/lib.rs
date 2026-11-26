/// Renvoie la version de l'application, lue dans `Cargo.toml`.
/// Sert aussi à vérifier que le front communique avec le Rust.
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Point d'entrée de l'application, appelé par `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!())
        .expect("impossible de démarrer l'application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_matches_cargo() {
        assert_eq!(app_version(), env!("CARGO_PKG_VERSION"));
    }
}
