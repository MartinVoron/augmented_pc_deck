//! Journal de l'application.
//!
//! Les messages (Rust et interface) sont écrits dans un fichier
//! `augmented-pc-deck.log`, dans le dossier des journaux de l'application :
//! `%LOCALAPPDATA%\fr.martinvoron.augmented-pc-deck\logs` sous Windows.
//! Quand le fichier dépasse [`MAX_FILE_SIZE`], il est archivé et un nouveau commence ;
//! seuls les [`KEPT_FILES`] derniers fichiers sont gardés.

use log::LevelFilter;
use tauri::plugin::TauriPlugin;
use tauri::Runtime;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

/// Taille maximale d'un fichier de journal : 1 Mo.
pub const MAX_FILE_SIZE: u128 = 1_000_000;

/// Nombre de fichiers de journal gardés après rotation.
pub const KEPT_FILES: usize = 5;

/// Niveau minimal enregistré : tout en développement, l'essentiel en version finale.
fn level() -> LevelFilter {
    if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}

/// Plugin de journal à installer au démarrage de l'application.
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_log::Builder::new()
        .clear_targets()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir {
                file_name: Some("augmented-pc-deck".into()),
            }),
        ])
        .level(level())
        .max_file_size(MAX_FILE_SIZE)
        .rotation_strategy(RotationStrategy::KeepSome(KEPT_FILES))
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .build()
}

/// Écrit dans le journal toute panique Rust, avant le comportement habituel.
pub fn log_panics() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panique : {info}");
        default_hook(info);
    }));
}
