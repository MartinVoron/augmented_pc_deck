//! Jetons de connexion (GitHub, Gmail, Claude), gardés hors des fichiers de l'application.
//!
//! Sous Windows, ils sont stockés dans le Gestionnaire d'identification
//! (Panneau de configuration > Gestionnaire d'identification > Informations
//! d'identification Windows), sous le nom `augmented-pc-deck`.
//!
//! Règles :
//! - un jeton n'est jamais écrit dans `config.json` ni dans le journal ;
//! - l'interface peut enregistrer ou supprimer un jeton, et savoir s'il existe,
//!   mais ne peut jamais le relire : seul le Rust s'en sert, via [`TokenStore::get`].

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};

/// Nom de service sous lequel les jetons apparaissent dans le Gestionnaire d'identification.
pub const SERVICE: &str = "augmented-pc-deck";

/// Service auquel un jeton donne accès.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    Github,
    Gmail,
    Claude,
}

impl TokenKind {
    pub const ALL: [TokenKind; 3] = [Self::Github, Self::Gmail, Self::Claude];

    /// Nom du compte dans le Gestionnaire d'identification.
    fn account(self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gmail => "gmail",
            Self::Claude => "claude",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Github => "GitHub",
            Self::Gmail => "Gmail",
            Self::Claude => "Claude",
        }
    }
}

/// Valeur d'un jeton. Son affichage (`{:?}`) est masqué, pour ne jamais le
/// retrouver dans le journal par erreur.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

/// Endroit où les jetons sont réellement rangés.
pub trait SecretBackend: Send + Sync {
    fn get(&self, account: &str) -> AppResult<Option<String>>;
    fn set(&self, account: &str, value: &str) -> AppResult<()>;
    fn delete(&self, account: &str) -> AppResult<()>;
}

/// Gestionnaire d'identification de Windows (via la bibliothèque `keyring`).
pub struct KeyringBackend;

fn keyring_error(action: &str, error: keyring::Error) -> AppError {
    // Le message d'erreur de `keyring` ne contient jamais le jeton lui-même.
    AppError::Other(format!(
        "impossible de {action} le jeton dans le Gestionnaire d'identification : {error}"
    ))
}

impl SecretBackend for KeyringBackend {
    fn get(&self, account: &str) -> AppResult<Option<String>> {
        let entry = keyring::Entry::new(SERVICE, account).map_err(|e| keyring_error("lire", e))?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(keyring_error("lire", e)),
        }
    }

    fn set(&self, account: &str, value: &str) -> AppResult<()> {
        let entry =
            keyring::Entry::new(SERVICE, account).map_err(|e| keyring_error("enregistrer", e))?;
        entry
            .set_password(value)
            .map_err(|e| keyring_error("enregistrer", e))
    }

    fn delete(&self, account: &str) -> AppResult<()> {
        let entry =
            keyring::Entry::new(SERVICE, account).map_err(|e| keyring_error("supprimer", e))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(keyring_error("supprimer", e)),
        }
    }
}

/// Magasin en mémoire, utilisé par les tests.
#[derive(Default)]
pub struct MemoryBackend(Mutex<HashMap<String, String>>);

impl SecretBackend for MemoryBackend {
    fn get(&self, account: &str) -> AppResult<Option<String>> {
        Ok(self.0.lock().unwrap().get(account).cloned())
    }

    fn set(&self, account: &str, value: &str) -> AppResult<()> {
        self.0.lock().unwrap().insert(account.into(), value.into());
        Ok(())
    }

    fn delete(&self, account: &str) -> AppResult<()> {
        self.0.lock().unwrap().remove(account);
        Ok(())
    }
}

/// Accès aux jetons pour toute l'application.
pub struct TokenStore {
    backend: Box<dyn SecretBackend>,
}

impl TokenStore {
    pub fn new(backend: impl SecretBackend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
        }
    }

    /// Magasin de Windows, utilisé par l'application.
    pub fn system() -> Self {
        Self::new(KeyringBackend)
    }

    /// Lit un jeton (pour les modules Rust uniquement).
    pub fn get(&self, kind: TokenKind) -> AppResult<Option<Secret>> {
        Ok(self.backend.get(kind.account())?.map(Secret))
    }

    pub fn set(&self, kind: TokenKind, value: &str) -> AppResult<()> {
        let value = value.trim();
        if value.is_empty() {
            return Err(AppError::InvalidInput(format!(
                "le jeton {} est vide",
                kind.label()
            )));
        }
        self.backend.set(kind.account(), value)?;
        log::info!("jeton {} enregistré", kind.label());
        Ok(())
    }

    pub fn delete(&self, kind: TokenKind) -> AppResult<()> {
        self.backend.delete(kind.account())?;
        log::info!("jeton {} supprimé", kind.label());
        Ok(())
    }

    /// Indique, pour chaque service, si un jeton est enregistré.
    pub fn status(&self) -> AppResult<HashMap<TokenKind, bool>> {
        TokenKind::ALL
            .into_iter()
            .map(|kind| Ok((kind, self.backend.get(kind.account())?.is_some())))
            .collect()
    }
}

/// Indique quels jetons sont enregistrés, sans jamais renvoyer leur valeur.
#[tauri::command]
pub fn tokens_status(store: State<'_, TokenStore>) -> AppResult<HashMap<TokenKind, bool>> {
    store.status()
}

/// Enregistre (ou remplace) un jeton.
#[tauri::command]
pub fn tokens_set(store: State<'_, TokenStore>, kind: TokenKind, value: String) -> AppResult<()> {
    store.set(kind, &value)
}

/// Supprime un jeton.
#[tauri::command]
pub fn tokens_delete(store: State<'_, TokenStore>, kind: TokenKind) -> AppResult<()> {
    store.delete(kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> TokenStore {
        TokenStore::new(MemoryBackend::default())
    }

    #[test]
    fn saves_reads_and_deletes_a_token() {
        let store = store();
        store.set(TokenKind::Github, "  ghp_exemple  ").unwrap();

        assert_eq!(
            store.get(TokenKind::Github).unwrap().unwrap().expose(),
            "ghp_exemple"
        );
        assert_eq!(store.get(TokenKind::Gmail).unwrap(), None);

        store.delete(TokenKind::Github).unwrap();
        assert_eq!(store.get(TokenKind::Github).unwrap(), None);
    }

    #[test]
    fn status_tells_which_tokens_exist() {
        let store = store();
        store.set(TokenKind::Claude, "jeton").unwrap();

        let status = store.status().unwrap();
        assert!(status[&TokenKind::Claude]);
        assert!(!status[&TokenKind::Github]);
        assert!(!status[&TokenKind::Gmail]);
    }

    #[test]
    fn status_sent_to_interface_contains_no_token() {
        let store = store();
        store.set(TokenKind::Github, "ghp_secret").unwrap();

        let json = serde_json::to_string(&store.status().unwrap()).unwrap();
        assert!(!json.contains("ghp_secret"));
        assert!(json.contains("\"github\":true"));
    }

    #[test]
    fn rejects_empty_token() {
        let error = store().set(TokenKind::Gmail, "   ").unwrap_err();
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn secret_is_hidden_when_printed() {
        let secret = Secret("ghp_secret".into());
        assert_eq!(format!("{secret:?}"), "Secret(***)");
    }

    #[test]
    fn deleting_a_missing_token_is_not_an_error() {
        assert!(store().delete(TokenKind::Gmail).is_ok());
    }
}
