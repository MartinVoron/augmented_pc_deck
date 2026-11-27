//! Client HTTPS commun aux modules qui appellent un service en ligne (GitHub, Gmail).

use std::sync::OnceLock;
use std::time::Duration;

use crate::error::AppError;

/// Délai maximal d'une requête : la tuile affiche une erreur plutôt que d'attendre.
const TIMEOUT: Duration = Duration::from_secs(15);

/// Client partagé (il garde ses connexions ouvertes d'un appel à l'autre).
pub fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(concat!("augmented-pc-deck/", env!("CARGO_PKG_VERSION")))
            .timeout(TIMEOUT)
            .build()
            .expect("impossible de préparer le client HTTPS")
    })
}

/// Erreur réseau (pas de connexion, délai dépassé…), avec un message lisible.
pub fn network_error(service: &str, error: reqwest::Error) -> AppError {
    let reason = if error.is_timeout() {
        "le délai d'attente est dépassé".to_string()
    } else if error.is_connect() {
        "pas de connexion à Internet".to_string()
    } else if error.is_decode() {
        "réponse illisible".to_string()
    } else {
        error.to_string()
    };
    AppError::Other(format!("{service} ne répond pas : {reason}"))
}
