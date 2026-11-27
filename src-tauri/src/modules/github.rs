//! GitHub : connexion avec un jeton personnel, puis activité à suivre.
//!
//! Le jeton est vérifié auprès de l'API (`GET /user`) avant d'être enregistré dans
//! le Gestionnaire d'identification ; l'interface ne le revoit jamais ensuite.
//!
//! L'activité tient en quatre requêtes faites en parallèle : trois recherches
//! (PR à relire, mes PR, issues assignées) et les notifications non lues.
//! L'interface les demande toutes les quelques minutes (réglable), bien en dessous
//! des limites de l'API (30 recherches par minute, 5 000 requêtes par heure).

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

/// Jeton GitHub enregistré, ou erreur claire s'il n'y en a pas.
fn stored_token(store: &TokenStore) -> AppResult<Secret> {
    store.get(TokenKind::Github)?.ok_or_else(|| {
        AppError::PermissionDenied("aucun compte GitHub connecté (voir les paramètres)".into())
    })
}

/// Nombre d'éléments gardés par liste.
const PER_SECTION: usize = 20;

/// Un élément à suivre : PR, issue ou notification.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityItem {
    pub title: String,
    /// Dépôt, sous la forme `propriétaire/nom`.
    pub repo: String,
    pub number: Option<u64>,
    /// Page à ouvrir dans le navigateur.
    pub url: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
    /// `pullRequest`, `issue`, ou le type d'une notification (`release`, `discussion`…).
    pub kind: String,
    /// Raison d'une notification (`review_requested`, `mention`…).
    pub reason: Option<String>,
}

/// Une liste de l'activité. Si sa requête échoue, les autres s'affichent quand même.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub items: Vec<ActivityItem>,
    /// Nombre total (il peut dépasser le nombre d'éléments renvoyés).
    pub total: u64,
    pub error: Option<String>,
}

impl Section {
    fn from_result(result: AppResult<(Vec<ActivityItem>, u64)>) -> Self {
        match result {
            Ok((items, total)) => Self {
                items,
                total,
                error: None,
            },
            Err(e) => Self {
                error: Some(e.to_string()),
                ..Self::default()
            },
        }
    }
}

/// Tout ce qui attend une action sur GitHub.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubActivity {
    /// PR où l'on m'a demandé une relecture.
    pub review_requests: Section,
    /// Mes PR ouvertes.
    pub my_pull_requests: Section,
    /// Issues ouvertes qui me sont assignées.
    pub assigned_issues: Section,
    /// Notifications non lues.
    pub notifications: Section,
}

#[derive(Deserialize)]
struct SearchResults {
    total_count: u64,
    items: Vec<SearchItem>,
}

#[derive(Deserialize)]
struct SearchItem {
    title: String,
    number: u64,
    html_url: String,
    repository_url: String,
    updated_at: String,
    pull_request: Option<serde_json::Value>,
}

impl From<SearchItem> for ActivityItem {
    fn from(item: SearchItem) -> Self {
        Self {
            repo: item
                .repository_url
                .trim_start_matches("https://api.github.com/repos/")
                .to_string(),
            kind: if item.pull_request.is_some() {
                "pullRequest".into()
            } else {
                "issue".into()
            },
            title: item.title,
            number: Some(item.number),
            url: item.html_url,
            updated_at: item.updated_at,
            reason: None,
        }
    }
}

#[derive(Deserialize)]
struct Notification {
    reason: String,
    updated_at: String,
    subject: Subject,
    repository: Repository,
}

#[derive(Deserialize)]
struct Subject {
    title: String,
    url: Option<String>,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
    html_url: String,
}

/// Page web d'un élément d'une notification, à partir de son adresse dans l'API
/// (`https://api.github.com/repos/o/r/pulls/12` → `https://github.com/o/r/pull/12`).
fn web_url(api_url: Option<&str>, repo_url: &str) -> (String, Option<u64>) {
    let Some(path) = api_url.and_then(|u| u.strip_prefix("https://api.github.com/repos/")) else {
        return (repo_url.to_string(), None);
    };
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        [owner, repo, "pulls", number] => (
            format!("https://github.com/{owner}/{repo}/pull/{number}"),
            number.parse().ok(),
        ),
        [owner, repo, "issues", number] => (
            format!("https://github.com/{owner}/{repo}/issues/{number}"),
            number.parse().ok(),
        ),
        // Versions, commits, discussions… : la page du dépôt suffit.
        _ => (repo_url.to_string(), None),
    }
}

impl From<Notification> for ActivityItem {
    fn from(n: Notification) -> Self {
        let (url, number) = web_url(n.subject.url.as_deref(), &n.repository.html_url);
        let mut kind = n.subject.kind;
        // GitHub écrit `PullRequest`, `Issue`… : on garde le style de l'interface.
        if let Some(first) = kind.get(0..1) {
            kind = first.to_lowercase() + &kind[1..];
        }
        Self {
            title: n.subject.title,
            repo: n.repository.full_name,
            number,
            url,
            updated_at: n.updated_at,
            kind,
            reason: Some(n.reason),
        }
    }
}

/// Recherche d'issues ou de PR ouvertes qui me concernent.
async fn search(token: &Secret, query: &str) -> AppResult<(Vec<ActivityItem>, u64)> {
    let path = format!(
        "/search/issues?q={}&sort=updated&order=desc&per_page={PER_SECTION}",
        query
            .replace(' ', "+")
            .replace(':', "%3A")
            .replace('@', "%40")
    );
    let (results, _): (SearchResults, _) = get_json(token, &path).await?;
    Ok((
        results.items.into_iter().map(Into::into).collect(),
        results.total_count,
    ))
}

/// Notifications non lues.
async fn notifications(token: &Secret) -> AppResult<(Vec<ActivityItem>, u64)> {
    let (list, _): (Vec<Notification>, _) = get_json(token, "/notifications?per_page=50").await?;
    let total = list.len() as u64;
    Ok((
        list.into_iter().take(PER_SECTION).map(Into::into).collect(),
        total,
    ))
}

/// Récupère tout ce qui attend une action sur GitHub.
#[tauri::command]
pub async fn github_activity(store: State<'_, TokenStore>) -> AppResult<GithubActivity> {
    let token = stored_token(&store)?;
    let (review_requests, my_pull_requests, assigned_issues, notifications) = tokio::join!(
        search(&token, "is:open is:pr archived:false review-requested:@me"),
        search(&token, "is:open is:pr archived:false author:@me"),
        search(&token, "is:open is:issue archived:false assignee:@me"),
        notifications(&token),
    );
    // Un jeton refusé l'est partout : une seule erreur claire plutôt que quatre.
    for result in [&review_requests, &my_pull_requests, &assigned_issues] {
        if let Err(AppError::PermissionDenied(message)) = result {
            if message.contains("refuse le jeton") {
                return Err(AppError::PermissionDenied(message.clone()));
            }
        }
    }
    Ok(GithubActivity {
        review_requests: Section::from_result(review_requests),
        my_pull_requests: Section::from_result(my_pull_requests),
        assigned_issues: Section::from_result(assigned_issues),
        notifications: Section::from_result(notifications),
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

    #[test]
    fn missing_token_is_a_clear_error() {
        let store = TokenStore::new(crate::tokens::MemoryBackend::default());
        let error = stored_token(&store).unwrap_err();
        assert!(error.to_string().contains("aucun compte GitHub"));
    }

    #[test]
    fn reads_search_results() {
        let results: SearchResults = serde_json::from_str(
            r#"{"total_count":42,"items":[
              {"title":"Ajouter le minuteur","number":12,"html_url":"https://github.com/o/r/pull/12",
               "repository_url":"https://api.github.com/repos/o/r","updated_at":"2026-11-26T10:00:00Z",
               "pull_request":{"url":"x"}},
              {"title":"Bogue","number":3,"html_url":"https://github.com/o/r/issues/3",
               "repository_url":"https://api.github.com/repos/o/r","updated_at":"2026-11-25T10:00:00Z"}
            ]}"#,
        )
        .unwrap();
        assert_eq!(results.total_count, 42);
        let items: Vec<ActivityItem> = results.items.into_iter().map(Into::into).collect();
        assert_eq!(items[0].repo, "o/r");
        assert_eq!(items[0].kind, "pullRequest");
        assert_eq!(items[1].kind, "issue");
        assert_eq!(items[1].number, Some(3));
    }

    #[test]
    fn turns_api_addresses_into_web_pages() {
        let repo = "https://github.com/o/r";
        assert_eq!(
            web_url(Some("https://api.github.com/repos/o/r/pulls/12"), repo),
            ("https://github.com/o/r/pull/12".into(), Some(12))
        );
        assert_eq!(
            web_url(Some("https://api.github.com/repos/o/r/issues/3"), repo),
            ("https://github.com/o/r/issues/3".into(), Some(3))
        );
        assert_eq!(
            web_url(Some("https://api.github.com/repos/o/r/releases/9"), repo),
            (repo.into(), None)
        );
        assert_eq!(web_url(None, repo), (repo.into(), None));
    }

    #[test]
    fn reads_a_notification() {
        let n: Notification = serde_json::from_str(
            r#"{"reason":"review_requested","updated_at":"2026-11-26T10:00:00Z","unread":true,
               "subject":{"title":"Corriger le lanceur","url":"https://api.github.com/repos/o/r/pulls/7","type":"PullRequest"},
               "repository":{"full_name":"o/r","html_url":"https://github.com/o/r"}}"#,
        )
        .unwrap();
        let item = ActivityItem::from(n);
        assert_eq!(item.kind, "pullRequest");
        assert_eq!(item.url, "https://github.com/o/r/pull/7");
        assert_eq!(item.reason.as_deref(), Some("review_requested"));
    }

    #[test]
    fn a_failed_list_keeps_its_error() {
        let section = Section::from_result(Err(AppError::Other("GitHub ne répond pas".into())));
        assert!(section.items.is_empty());
        assert_eq!(section.error.as_deref(), Some("GitHub ne répond pas"));
    }
}
