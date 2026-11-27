//! Connexion OAuth 2.0 à Google, pour une application de bureau.
//!
//! Déroulé (flux « code d'autorisation » avec PKCE, recommandé par Google) :
//! 1. un petit serveur s'ouvre sur `127.0.0.1`, sur un port libre choisi par Windows ;
//! 2. le navigateur ouvre la page de connexion de Google ;
//! 3. après l'accord de l'utilisateur, Google renvoie le navigateur vers ce serveur
//!    avec un code à usage unique ;
//! 4. le code est échangé contre un jeton d'accès (1 heure) et un jeton de
//!    rafraîchissement (durable), qui permet d'obtenir de nouveaux jetons d'accès.

use std::time::Duration;

use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use url::Url;

use crate::error::{AppError, AppResult};
use crate::http::{client, network_error};

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";

/// Seul droit demandé : lire les e-mails, sans pouvoir en envoyer ni en supprimer.
pub const SCOPE: &str = "https://www.googleapis.com/auth/gmail.readonly";

/// Temps laissé pour se connecter dans le navigateur.
const LOGIN_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Identifiant OAuth créé dans la console Google Cloud (type « Application de bureau »).
#[derive(Clone)]
pub struct Client {
    pub id: String,
    /// Pour une application de bureau, Google le considère comme non confidentiel,
    /// mais il reste rangé dans le Gestionnaire d'identification, avec le reste.
    pub secret: String,
}

/// Jetons renvoyés par Google.
#[derive(Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub expires_in: u64,
    /// Seulement à la première connexion (pas lors d'un rafraîchissement).
    pub refresh_token: Option<String>,
}

#[derive(Deserialize)]
struct TokenError {
    error: String,
    error_description: Option<String>,
}

/// Chaîne aléatoire sûre pour une adresse (base64 sans `+`, `/` ni `=`).
fn random_string(bytes: usize) -> AppResult<String> {
    let mut buffer = vec![0u8; bytes];
    getrandom::fill(&mut buffer)
        .map_err(|e| AppError::Other(format!("aléa indisponible : {e}")))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buffer))
}

/// Empreinte envoyée à Google ; seul ce programme connaît le `verifier` d'origine.
fn pkce_challenge(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Adresse de la page de connexion de Google.
fn authorization_url(client_id: &str, redirect_uri: &str, challenge: &str, state: &str) -> Url {
    Url::parse_with_params(
        AUTH_URL,
        &[
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("response_type", "code"),
            ("scope", SCOPE),
            ("code_challenge", challenge),
            ("code_challenge_method", "S256"),
            ("state", state),
            // Demande un jeton de rafraîchissement, même après une première connexion.
            ("access_type", "offline"),
            ("prompt", "consent"),
        ],
    )
    .expect("adresse de connexion Google invalide")
}

/// Ce que Google a renvoyé au navigateur, lu dans la première ligne de la requête
/// (`GET /?code=…&state=… HTTP/1.1`). `None` pour une autre requête (`/favicon.ico`).
fn parse_callback(request_line: &str) -> Option<Result<(String, String), String>> {
    let path = request_line.split_whitespace().nth(1)?;
    let url = Url::parse(&format!("http://127.0.0.1{path}")).ok()?;
    if url.path() != "/" {
        return None;
    }
    let param = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    if let Some(error) = param("error") {
        return Some(Err(error));
    }
    Some(Ok((param("code")?, param("state")?)))
}

/// Page affichée dans le navigateur à la fin de la connexion.
fn result_page(success: bool) -> String {
    let (title, text) = if success {
        (
            "Connexion réussie",
            "Augmented PC Deck peut maintenant lire vos e-mails non lus. Vous pouvez fermer cet onglet.",
        )
    } else {
        (
            "Connexion annulée",
            "La connexion à Gmail n'a pas abouti. Vous pouvez fermer cet onglet et réessayer depuis l'application.",
        )
    };
    let body = format!(
        "<!doctype html><html lang=\"fr\"><meta charset=\"utf-8\"><title>{title}</title>\
         <body style=\"font-family:Segoe UI,sans-serif;background:#14161b;color:#e6e8ee;\
         display:grid;place-items:center;height:100vh;margin:0\"><div style=\"text-align:center\">\
         <h1>{title}</h1><p>{text}</p></div></body></html>"
    );
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// Attend le retour du navigateur sur le serveur local, et renvoie le code reçu.
async fn wait_for_code(listener: &TcpListener, expected_state: &str) -> AppResult<String> {
    loop {
        let (mut stream, _) = listener.accept().await?;
        let mut buffer = vec![0u8; 8192];
        let read = stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..read]);
        let line = request.lines().next().unwrap_or_default();
        let Some(result) = parse_callback(line) else {
            let _ = stream
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await;
            continue;
        };
        let accepted = matches!(&result, Ok((_, state)) if state == expected_state);
        let _ = stream.write_all(result_page(accepted).as_bytes()).await;
        let _ = stream.shutdown().await;
        return match result {
            Ok((code, _)) if accepted => Ok(code),
            Ok(_) => Err(AppError::PermissionDenied(
                "la réponse de Google ne correspond pas à cette connexion".into(),
            )),
            Err(error) if error == "access_denied" => Err(AppError::PermissionDenied(
                "l'accès à Gmail a été refusé dans le navigateur".into(),
            )),
            Err(error) => Err(AppError::Other(format!(
                "Google a refusé la connexion : {error}"
            ))),
        };
    }
}

/// Envoie un formulaire au serveur de jetons de Google.
async fn token_request(form: &[(&str, &str)]) -> AppResult<Tokens> {
    let response = client()
        .post(TOKEN_URL)
        .form(form)
        .send()
        .await
        .map_err(|e| network_error("Google", e))?;
    if response.status().is_success() {
        return response
            .json()
            .await
            .map_err(|e| network_error("Google", e));
    }
    let error: TokenError = response
        .json()
        .await
        .map_err(|e| network_error("Google", e))?;
    Err(token_error(error))
}

fn token_error(error: TokenError) -> AppError {
    match error.error.as_str() {
        "invalid_grant" => AppError::PermissionDenied(
            "Google a retiré l'accès à Gmail (mot de passe changé, accès révoqué ou application \
             en mode test depuis plus de 7 jours) : reconnectez-vous dans les paramètres"
                .into(),
        ),
        "invalid_client" | "unauthorized_client" => AppError::InvalidInput(
            "l'identifiant OAuth est incorrect : vérifiez l'ID client et le code secret".into(),
        ),
        _ => AppError::Other(format!(
            "Google a refusé la demande : {}",
            error.error_description.unwrap_or(error.error)
        )),
    }
}

/// Connexion complète : ouvre le navigateur et attend l'accord de l'utilisateur.
/// `open` reçoit l'adresse de la page de connexion à ouvrir.
pub async fn login(client: &Client, open: impl FnOnce(&str) -> AppResult<()>) -> AppResult<Tokens> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let redirect_uri = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let verifier = random_string(48)?;
    let state = random_string(16)?;
    let url = authorization_url(
        &client.id,
        &redirect_uri,
        &pkce_challenge(&verifier),
        &state,
    );

    open(url.as_str())?;
    let code = tokio::time::timeout(LOGIN_TIMEOUT, wait_for_code(&listener, &state))
        .await
        .map_err(|_| {
            AppError::Other(
                "connexion abandonnée : aucune réponse du navigateur en 5 minutes".into(),
            )
        })??;

    token_request(&[
        ("code", &code),
        ("client_id", &client.id),
        ("client_secret", &client.secret),
        ("redirect_uri", &redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", &verifier),
    ])
    .await
}

/// Obtient un nouveau jeton d'accès à partir du jeton de rafraîchissement.
pub async fn refresh(client: &Client, refresh_token: &str) -> AppResult<Tokens> {
    token_request(&[
        ("client_id", &client.id),
        ("client_secret", &client.secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ])
    .await
}

/// Révoque l'accès auprès de Google (au mieux : une erreur n'empêche pas la déconnexion).
pub async fn revoke(token: &str) {
    let result = client()
        .post(REVOKE_URL)
        .form(&[("token", token)])
        .send()
        .await;
    if let Err(e) = result {
        log::warn!(
            "révocation de l'accès Gmail impossible : {}",
            network_error("Google", e)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_the_rfc_example() {
        // Exemple de la RFC 7636, annexe B.
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn random_strings_are_url_safe_and_different() {
        let a = random_string(48).unwrap();
        let b = random_string(48).unwrap();
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn authorization_url_asks_for_read_only_offline_access() {
        let url = authorization_url("id-client", "http://127.0.0.1:5000", "empreinte", "etat");
        let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params["scope"], SCOPE);
        assert_eq!(params["access_type"], "offline");
        assert_eq!(params["code_challenge_method"], "S256");
        assert_eq!(params["redirect_uri"], "http://127.0.0.1:5000");
        assert_eq!(params["state"], "etat");
    }

    #[test]
    fn reads_the_code_sent_back_by_google() {
        assert_eq!(
            parse_callback("GET /?state=abc&code=4%2F0Ab&scope=x HTTP/1.1"),
            Some(Ok(("4/0Ab".to_string(), "abc".to_string())))
        );
        assert_eq!(
            parse_callback("GET /?error=access_denied&state=abc HTTP/1.1"),
            Some(Err("access_denied".to_string()))
        );
        assert_eq!(parse_callback("GET /favicon.ico HTTP/1.1"), None);
        assert_eq!(parse_callback(""), None);
    }

    #[test]
    fn explains_a_revoked_access() {
        let error = token_error(TokenError {
            error: "invalid_grant".into(),
            error_description: None,
        });
        assert_eq!(error.code(), "permission_denied");
        assert!(error.to_string().contains("reconnectez-vous"));
    }

    #[tokio::test]
    async fn local_server_returns_the_code_and_a_page() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let browser = tokio::spawn(async move {
            // Le navigateur demande d'abord l'icône, puis la vraie adresse.
            for path in ["/favicon.ico", "/?code=le-code&state=etat"] {
                let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
                    .await
                    .unwrap();
                stream
                    .write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
                    .await
                    .unwrap();
                let mut page = String::new();
                stream.read_to_string(&mut page).await.unwrap();
                if path.starts_with("/?") {
                    return page;
                }
            }
            unreachable!()
        });
        let code = wait_for_code(&listener, "etat").await.unwrap();
        assert_eq!(code, "le-code");
        assert!(browser.await.unwrap().contains("Connexion réussie"));
    }

    #[tokio::test]
    async fn local_server_rejects_a_foreign_answer() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap();
            stream
                .write_all(b"GET /?code=c&state=autre HTTP/1.1\r\n\r\n")
                .await
                .unwrap();
        });
        let error = wait_for_code(&listener, "etat").await.unwrap_err();
        assert_eq!(error.code(), "permission_denied");
    }
}
