//! Un module par fonctionnalité de l'application.
//!
//! Convention pour les commandes Tauri :
//! - une commande s'appelle `<module>_<action>` en anglais (ex. `timers_start`),
//!   sauf les commandes générales du module `app` ;
//! - elle renvoie toujours un [`crate::error::AppResult`] ;
//! - elle est déclarée dans `lib.rs`, dans `tauri::generate_handler!`.

pub mod app;
pub mod claude;
pub mod github;
pub mod gmail;
pub mod launcher;
pub mod music;
pub mod processes;
pub mod settings;
pub mod timers;
pub mod todos;
