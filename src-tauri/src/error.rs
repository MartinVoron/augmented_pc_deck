//! Erreurs communes à toute l'application.
//!
//! Chaque commande Tauri renvoie un [`AppResult`]. En cas d'échec, l'interface reçoit
//! un objet JSON `{ "code": "...", "message": "..." }` :
//! - `code` est stable et sert au code de l'interface (par exemple `not_found`) ;
//! - `message` est une phrase en français, lisible telle quelle par l'utilisateur.

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

/// Résultat renvoyé par toutes les commandes Tauri.
pub type AppResult<T> = Result<T, AppError>;

/// Erreur renvoyée à l'interface.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// L'élément demandé n'existe pas (fichier, processus, tâche…).
    #[error("{0} est introuvable")]
    NotFound(String),

    /// Les données envoyées par l'interface sont incorrectes.
    #[error("donnée invalide : {0}")]
    InvalidInput(String),

    /// Windows ou un service extérieur a refusé l'opération.
    #[error("accès refusé : {0}")]
    PermissionDenied(String),

    /// Erreur de lecture ou d'écriture sur le disque.
    #[error("erreur de fichier : {0}")]
    Io(#[from] std::io::Error),

    /// Toute autre erreur, avec un message déjà rédigé.
    #[error("{0}")]
    Other(String),
}

impl AppError {
    /// Code stable, en anglais, utilisé par l'interface pour réagir à l'erreur.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::InvalidInput(_) => "invalid_input",
            Self::PermissionDenied(_) => "permission_denied",
            Self::Io(_) => "io",
            Self::Other(_) => "other",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_code_and_message() {
        let error = AppError::NotFound("Le fichier notes.txt".into());
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "not_found");
        assert_eq!(json["message"], "Le fichier notes.txt est introuvable");
    }
}
