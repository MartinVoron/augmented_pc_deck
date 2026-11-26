//! Informations générales sur l'application.

use crate::error::AppResult;

/// Renvoie la version de l'application, lue dans `Cargo.toml`.
/// Sert aussi à vérifier que l'interface communique avec le Rust.
#[tauri::command]
pub fn app_version() -> AppResult<String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_matches_cargo() {
        assert_eq!(app_version().unwrap(), env!("CARGO_PKG_VERSION"));
    }
}
