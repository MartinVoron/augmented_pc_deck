//! Gmail : connexion OAuth 2.0 en lecture seule.
//!
//! Tout ce qui permet d'accéder à la boîte (identifiant OAuth, jeton de
//! rafraîchissement, adresse du compte) est rangé ensemble, en JSON, dans l'entrée
//! `gmail` du Gestionnaire d'identification. Le jeton d'accès, valable une heure,
//! reste seulement en mémoire et est renouvelé automatiquement.

mod inbox;
mod oauth;

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime, State};
use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::http::{client, network_error};
use crate::tokens::{Secret, TokenKind, TokenStore};

const PROFILE_URL: &str = "https://gmail.googleapis.com/gmail/v1/users/me/profile";

/// Marge avant l'expiration d'un jeton d'accès, pour ne pas l'utiliser trop tard.
const EXPIRY_MARGIN: Duration = Duration::from_secs(60);

/// Ce qui est rangé dans le Gestionnaire d'identification.
#[derive(Clone, Serialize, Deserialize)]
struct Credentials {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    email: String,
}

/// Affichage masqué : ces informations ne doivent jamais finir dans le journal.
impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Credentials(***)")
    }
}

impl Credentials {
    fn client(&self) -> oauth::Client {
        oauth::Client {
            id: self.client_id.clone(),
            secret: self.client_secret.clone(),
        }
    }
}

/// Compte Gmail connecté.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailAccount {
    pub email: String,
}

pub use inbox::Inbox;

/// Ce que le module garde en mémoire : le jeton d'accès (avec son heure
/// d'expiration) et les derniers messages lus.
#[derive(Default)]
pub struct GmailState {
    access: Mutex<Option<(Secret, Instant)>>,
    inbox: Mutex<inbox::InboxCache>,
}

fn load(store: &TokenStore) -> AppResult<Option<Credentials>> {
    let Some(secret) = store.get(TokenKind::Gmail)? else {
        return Ok(None);
    };
    serde_json::from_str(secret.expose())
        .map(Some)
        .map_err(|_| {
            AppError::Other(
                "les informations de connexion Gmail sont illisibles : reconnectez-vous".into(),
            )
        })
}

fn save(store: &TokenStore, credentials: &Credentials) -> AppResult<()> {
    let json = serde_json::to_string(credentials)
        .map_err(|e| AppError::Other(format!("connexion Gmail impossible à enregistrer : {e}")))?;
    store.set(TokenKind::Gmail, &json)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    email_address: String,
}

/// Adresse e-mail du compte connecté.
async fn fetch_email(access_token: &str) -> AppResult<String> {
    let response = client()
        .get(PROFILE_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| network_error("Gmail", e))?;
    if !response.status().is_success() {
        return Err(api_error(response.status()));
    }
    let profile: Profile = response
        .json()
        .await
        .map_err(|e| network_error("Gmail", e))?;
    Ok(profile.email_address)
}

/// Traduit un refus de l'API Gmail en erreur lisible.
pub(crate) fn api_error(status: reqwest::StatusCode) -> AppError {
    match status.as_u16() {
        401 => AppError::PermissionDenied("Gmail refuse l'accès : reconnectez-vous".into()),
        403 => AppError::PermissionDenied(
            "Gmail refuse l'accès : vérifiez que l'API Gmail est activée dans la console Google Cloud"
                .into(),
        ),
        429 => AppError::Other("limite de l'API Gmail atteinte : nouvel essai plus tard".into()),
        _ => AppError::Other(format!("Gmail a répondu « {status} »")),
    }
}

/// Jeton d'accès valide, renouvelé si besoin.
pub(crate) async fn access_token(state: &GmailState, store: &TokenStore) -> AppResult<Secret> {
    let mut access = state.access.lock().await;
    if let Some((token, expires)) = access.as_ref() {
        if Instant::now() + EXPIRY_MARGIN < *expires {
            return Ok(token.clone());
        }
    }
    let credentials = load(store)?.ok_or_else(|| {
        AppError::PermissionDenied("aucun compte Gmail connecté (voir les paramètres)".into())
    })?;
    let tokens = oauth::refresh(&credentials.client(), &credentials.refresh_token).await?;
    let token = Secret::new(tokens.access_token);
    *access = Some((
        token.clone(),
        Instant::now() + Duration::from_secs(tokens.expires_in),
    ));
    Ok(token)
}

/// Connecte un compte Gmail : ouvre la page de connexion de Google dans le navigateur
/// et attend l'accord de l'utilisateur (5 minutes au plus).
/// Sans ID client ni code secret, reprend ceux déjà enregistrés (reconnexion).
#[tauri::command]
pub async fn gmail_connect(
    state: State<'_, GmailState>,
    store: State<'_, TokenStore>,
    client_id: String,
    client_secret: String,
) -> AppResult<GmailAccount> {
    let (client_id, client_secret) = (client_id.trim(), client_secret.trim());
    let client = if client_id.is_empty() && client_secret.is_empty() {
        // Reconnexion : on reprend l'identifiant OAuth déjà enregistré.
        load(&store)?.map(|c| c.client()).ok_or_else(|| {
            AppError::InvalidInput("l'ID client et le code secret sont nécessaires".into())
        })?
    } else {
        oauth::Client {
            id: client_id.to_string(),
            secret: client_secret.to_string(),
        }
    };
    if client.id.is_empty() || client.secret.is_empty() {
        return Err(AppError::InvalidInput(
            "l'ID client et le code secret sont nécessaires".into(),
        ));
    }
    if !client.id.ends_with(".apps.googleusercontent.com") {
        return Err(AppError::InvalidInput(
            "l'ID client se termine par « .apps.googleusercontent.com »".into(),
        ));
    }
    let tokens = oauth::login(&client, |url| {
        open::that_detached(url)
            .map_err(|e| AppError::Other(format!("impossible d'ouvrir le navigateur : {e}")))
    })
    .await?;
    let refresh_token = tokens.refresh_token.ok_or_else(|| {
        AppError::Other("Google n'a pas renvoyé de jeton de rafraîchissement : réessayez".into())
    })?;
    let email = fetch_email(&tokens.access_token).await?;
    save(
        &store,
        &Credentials {
            client_id: client.id,
            client_secret: client.secret,
            refresh_token,
            email: email.clone(),
        },
    )?;
    *state.access.lock().await = Some((
        Secret::new(tokens.access_token),
        Instant::now() + Duration::from_secs(tokens.expires_in),
    ));
    state.inbox.lock().await.clear();
    log::info!("compte Gmail connecté");
    Ok(GmailAccount { email })
}

/// Compte connecté, ou `None`. Ne fait aucune requête : l'adresse est enregistrée.
#[tauri::command]
pub fn gmail_account(store: State<'_, TokenStore>) -> AppResult<Option<GmailAccount>> {
    Ok(load(&store)?.map(|c| GmailAccount { email: c.email }))
}

/// Déconnecte le compte : l'accès est révoqué chez Google et tout est effacé.
#[tauri::command]
pub async fn gmail_disconnect(
    state: State<'_, GmailState>,
    store: State<'_, TokenStore>,
) -> AppResult<()> {
    // Une connexion illisible doit pouvoir être effacée quand même.
    if let Ok(Some(credentials)) = load(&store) {
        oauth::revoke(&credentials.refresh_token).await;
    }
    *state.access.lock().await = None;
    state.inbox.lock().await.clear();
    store.delete(TokenKind::Gmail)
}

/// Nombre de non lus et derniers messages de la boîte de réception.
/// Avec `notify`, une notification Windows signale chaque message arrivé depuis
/// l'appel précédent (rien au premier appel).
#[tauri::command]
pub async fn gmail_unread<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, GmailState>,
    store: State<'_, TokenStore>,
    notify: bool,
) -> AppResult<Inbox> {
    let token = access_token(&state, &store).await?;
    let email = load(&store)?.map(|c| c.email).unwrap_or_default();
    let (inbox, arrivals) = inbox::fetch(&token, &email, &state.inbox).await?;
    if notify {
        inbox::notify_arrivals(&app, &arrivals);
    }
    Ok(inbox)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::MemoryBackend;

    fn credentials() -> Credentials {
        Credentials {
            client_id: "123.apps.googleusercontent.com".into(),
            client_secret: "GOCSPX-secret".into(),
            refresh_token: "1//rafraichir".into(),
            email: "moi@gmail.com".into(),
        }
    }

    #[test]
    fn credentials_are_saved_together_and_read_back() {
        let store = TokenStore::new(MemoryBackend::default());
        assert!(load(&store).unwrap().is_none());

        save(&store, &credentials()).unwrap();
        let read = load(&store).unwrap().unwrap();
        assert_eq!(read.email, "moi@gmail.com");
        assert_eq!(read.refresh_token, "1//rafraichir");
    }

    #[test]
    fn unreadable_credentials_ask_to_reconnect() {
        let store = TokenStore::new(MemoryBackend::default());
        store.set(TokenKind::Gmail, "pas du json").unwrap();
        assert!(load(&store)
            .unwrap_err()
            .to_string()
            .contains("reconnectez-vous"));
    }

    #[test]
    fn explains_api_refusals() {
        assert_eq!(
            api_error(reqwest::StatusCode::UNAUTHORIZED).code(),
            "permission_denied"
        );
        assert!(api_error(reqwest::StatusCode::FORBIDDEN)
            .to_string()
            .contains("API Gmail"));
    }

    #[tokio::test]
    async fn without_account_the_access_token_is_a_clear_error() {
        let store = TokenStore::new(MemoryBackend::default());
        let error = access_token(&GmailState::default(), &store)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("aucun compte Gmail"));
    }

    #[tokio::test]
    async fn a_fresh_access_token_is_reused() {
        let store = TokenStore::new(MemoryBackend::default());
        let state = GmailState::default();
        *state.access.lock().await = Some((
            Secret::new("ya29.acces"),
            Instant::now() + Duration::from_secs(3600),
        ));
        let token = access_token(&state, &store).await.unwrap();
        assert_eq!(token.expose(), "ya29.acces");
    }
}
