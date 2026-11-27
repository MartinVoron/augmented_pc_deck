//! GitHub : connexion avec un jeton personnel.
//!
//! Le jeton est vérifié auprès de l'API (`GET /user`) avant d'être enregistré dans
//! le Gestionnaire d'identification ; l'interface ne le revoit jamais ensuite.

use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::http::{client, network_error};
use crate::tokens::{Secret, TokenKind, TokenStore};

const API: &str = "https://api.github.com";

/// Compte GitHub connecté, tel qu'affiché dans les paramètres.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubAccount {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
    /// Droits d'un jeton classique (`None` pour un jeton à accès précis).
    pub scopes: Option<Vec<String>>,
}

/// Réponse de `GET /user` (seuls les champs utiles).
#[derive(Deserialize)]
struct User {
    login: String,
    name: Option<String>,
    avatar_url: String,
    html_url: String,
}

/// Droits annoncés par GitHub pour un jeton classique (en-tête `x-oauth-scopes`).
fn scopes(headers: &HeaderMap) -> Option<Vec<String>> {
    let value = headers.get("x-oauth-scopes")?.to_str().ok()?;
    Some(
        value
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect(),
    )
}

/// Traduit un refus de GitHub en erreur lisible.
fn check_status(status: StatusCode, headers: &HeaderMap) -> AppResult<()> {
    if status.is_success() {
        return Ok(());
    }
    let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
    match status {
        StatusCode::UNAUTHORIZED => Err(AppError::PermissionDenied(
            "GitHub refuse le jeton : il est incorrect, révoqué ou expiré".into(),
        )),
        StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS
            if header("x-ratelimit-remaining") == Some("0") || header("retry-after").is_some() =>
        {
            Err(AppError::Other(
                "limite de l'API GitHub atteinte : nouvel essai dans quelques minutes".into(),
            ))
        }
        StatusCode::FORBIDDEN | StatusCode::NOT_FOUND => Err(AppError::PermissionDenied(
            "le jeton GitHub n'a pas les droits nécessaires".into(),
        )),
        _ => Err(AppError::Other(format!("GitHub a répondu « {status} »"))),
    }
}

/// Appelle l'API GitHub et renvoie la réponse JSON avec ses en-têtes.
pub(crate) async fn get_json<T: DeserializeOwned>(
    token: &Secret,
    path: &str,
) -> AppResult<(T, HeaderMap)> {
    let response = client()
        .get(format!("{API}{path}"))
        .header(AUTHORIZATION, format!("Bearer {}", token.expose()))
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| network_error("GitHub", e))?;
    let headers = response.headers().clone();
    check_status(response.status(), &headers)?;
    let body = response
        .json()
        .await
        .map_err(|e| network_error("GitHub", e))?;
    Ok((body, headers))
}

async fn fetch_account(token: &Secret) -> AppResult<GithubAccount> {
    let (user, headers): (User, _) = get_json(token, "/user").await?;
    Ok(GithubAccount {
        login: user.login,
        name: user.name.filter(|n| !n.is_empty()),
        avatar_url: user.avatar_url,
        html_url: user.html_url,
        scopes: scopes(&headers),
    })
}

/// Vérifie un jeton auprès de GitHub, puis l'enregistre s'il est accepté.
#[tauri::command]
pub async fn github_connect(
    store: State<'_, TokenStore>,
    token: String,
) -> AppResult<GithubAccount> {
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(AppError::InvalidInput("le jeton GitHub est vide".into()));
    }
    let account = fetch_account(&Secret::new(token.clone())).await?;
    store.set(TokenKind::Github, &token)?;
    log::info!("compte GitHub connecté : {}", account.login);
    Ok(account)
}

/// Compte connecté, ou `None` si aucun jeton n'est enregistré.
#[tauri::command]
pub async fn github_account(store: State<'_, TokenStore>) -> AppResult<Option<GithubAccount>> {
    match store.get(TokenKind::Github)? {
        Some(token) => fetch_account(&token).await.map(Some),
        None => Ok(None),
    }
}

/// Déconnecte le compte : le jeton est supprimé du Gestionnaire d'identification.
#[tauri::command]
pub fn github_disconnect(store: State<'_, TokenStore>) -> AppResult<()> {
    store.delete(TokenKind::Github)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_static(value));
        }
        map
    }

    #[test]
    fn reads_the_scopes_of_a_classic_token() {
        let map = headers(&[("x-oauth-scopes", "repo, notifications")]);
        assert_eq!(
            scopes(&map),
            Some(vec!["repo".to_string(), "notifications".to_string()])
        );
        assert_eq!(scopes(&HeaderMap::new()), None);
    }

    #[test]
    fn explains_a_refused_token() {
        let error = check_status(StatusCode::UNAUTHORIZED, &HeaderMap::new()).unwrap_err();
        assert_eq!(error.code(), "permission_denied");
        assert!(error.to_string().contains("refuse le jeton"));
    }

    #[test]
    fn tells_a_rate_limit_from_missing_rights() {
        let limited = headers(&[("x-ratelimit-remaining", "0")]);
        let error = check_status(StatusCode::FORBIDDEN, &limited).unwrap_err();
        assert!(error.to_string().contains("limite"));

        let error = check_status(StatusCode::FORBIDDEN, &HeaderMap::new()).unwrap_err();
        assert!(error.to_string().contains("droits"));
    }

    #[test]
    fn accepts_a_success() {
        assert!(check_status(StatusCode::OK, &HeaderMap::new()).is_ok());
    }

    #[test]
    fn reads_the_user() {
        let user: User = serde_json::from_str(
            r#"{"login":"octocat","name":null,"avatar_url":"https://a/1","html_url":"https://github.com/octocat","id":1}"#,
        )
        .unwrap();
        assert_eq!(user.login, "octocat");
        assert_eq!(user.name, None);
    }
}
