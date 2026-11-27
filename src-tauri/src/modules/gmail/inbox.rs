//! E-mails non lus de la boîte de réception.
//!
//! Chaque actualisation fait deux requêtes (le nombre de non lus, puis la liste des
//! derniers), plus une par nouveau message : l'expéditeur et l'objet d'un message
//! déjà vu sont gardés en mémoire. C'est très loin des limites de l'API Gmail.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::error::AppResult;
use crate::http::{client, network_error};
use crate::notifications::{self, Notification, Sound};
use crate::tokens::Secret;

use super::api_error;

const API: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

/// Nombre de messages affichés dans la tuile.
pub const MAX_MESSAGES: usize = 10;

/// Un e-mail non lu, tel qu'affiché dans la tuile.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    /// Nom de l'expéditeur (ou son adresse s'il n'a pas de nom).
    pub from: String,
    pub subject: String,
    /// Début du texte.
    pub snippet: String,
    /// Date de réception, en millisecondes depuis 1970.
    pub received_at: u64,
    /// Page du message dans Gmail.
    pub url: String,
}

/// Ce que la tuile reçoit.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inbox {
    pub email: String,
    /// Nombre total de non lus dans la boîte de réception.
    pub unread: u64,
    /// Les derniers non lus, du plus récent au plus ancien.
    pub messages: Vec<Message>,
}

/// Mémoire du module : messages déjà lus dans l'API, et déjà signalés.
#[derive(Default)]
pub struct InboxCache {
    messages: HashMap<String, Message>,
    /// `None` tant que la première liste n'est pas arrivée : rien n'est signalé
    /// au démarrage, seulement les messages arrivés ensuite.
    seen: Option<HashSet<String>>,
}

impl InboxCache {
    /// Retient les messages affichés, et renvoie ceux qui viennent d'arriver.
    fn new_arrivals(&mut self, messages: &[Message]) -> Vec<Message> {
        let ids = messages.iter().map(|m| m.id.clone());
        let Some(seen) = &mut self.seen else {
            self.seen = Some(ids.collect());
            return Vec::new();
        };
        messages
            .iter()
            .filter(|m| seen.insert(m.id.clone()))
            .cloned()
            .collect()
    }

    /// Oublie tout (changement de compte).
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Label {
    #[serde(default)]
    messages_unread: u64,
}

#[derive(Deserialize)]
struct MessageList {
    #[serde(default)]
    messages: Vec<MessageRef>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageRef {
    id: String,
    thread_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageMetadata {
    #[serde(default)]
    snippet: String,
    #[serde(default)]
    internal_date: String,
    payload: Payload,
}

#[derive(Deserialize)]
struct Payload {
    #[serde(default)]
    headers: Vec<Header>,
}

#[derive(Deserialize)]
struct Header {
    name: String,
    value: String,
}

async fn get_json<T: serde::de::DeserializeOwned>(token: &str, url: &str) -> AppResult<T> {
    let response = client()
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| network_error("Gmail", e))?;
    if !response.status().is_success() {
        return Err(api_error(response.status()));
    }
    response.json().await.map_err(|e| network_error("Gmail", e))
}

/// Nom lisible d'un expéditeur : `"Jean Dupont" <jean@exemple.fr>` → `Jean Dupont`.
fn sender_name(from: &str) -> String {
    let from = from.trim();
    match from.rfind('<') {
        Some(start) if start > 0 => {
            let name = from[..start].trim().trim_matches('"').trim();
            if name.is_empty() {
                from[start..].trim_matches(['<', '>']).to_string()
            } else {
                name.to_string()
            }
        }
        _ => from.trim_matches(['<', '>']).to_string(),
    }
}

/// Le texte de Gmail contient des entités HTML (`&#39;`, `&amp;`…).
fn decode_entities(text: &str) -> String {
    text.replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

/// Page d'une conversation dans Gmail, sur le bon compte.
fn message_url(email: &str, thread_id: &str) -> String {
    format!("https://mail.google.com/mail/u/?authuser={email}#inbox/{thread_id}")
}

fn to_message(id: String, thread_id: &str, email: &str, meta: MessageMetadata) -> Message {
    let header = |name: &str| {
        meta.payload
            .headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.clone())
            .unwrap_or_default()
    };
    let subject = header("Subject");
    Message {
        id,
        from: sender_name(&header("From")),
        subject: if subject.trim().is_empty() {
            "(sans objet)".into()
        } else {
            subject
        },
        snippet: decode_entities(&meta.snippet),
        received_at: meta.internal_date.parse().unwrap_or(0),
        url: message_url(email, thread_id),
    }
}

/// Lit l'expéditeur, l'objet et le début d'un message.
async fn fetch_message(token: String, email: String, reference: MessageRef) -> AppResult<Message> {
    let url = format!(
        "{API}/messages/{}?format=metadata&metadataHeaders=From&metadataHeaders=Subject",
        reference.id
    );
    let meta: MessageMetadata = get_json(&token, &url).await?;
    Ok(to_message(reference.id, &reference.thread_id, &email, meta))
}

/// Nombre de non lus et derniers messages. Les messages déjà connus ne sont pas relus.
pub async fn fetch(
    token: &Secret,
    email: &str,
    cache: &tokio::sync::Mutex<InboxCache>,
) -> AppResult<(Inbox, Vec<Message>)> {
    let token = token.expose();
    let label_url = format!("{API}/labels/INBOX");
    let list_url = format!("{API}/messages?q=is%3Aunread+in%3Ainbox&maxResults={MAX_MESSAGES}");
    let (label, list) = tokio::join!(
        get_json::<Label>(token, &label_url),
        get_json::<MessageList>(token, &list_url),
    );
    let (label, list) = (label?, list?);

    let known: HashMap<String, Message> = {
        let cache = cache.lock().await;
        list.messages
            .iter()
            .filter_map(|r| cache.messages.get(&r.id).map(|m| (r.id.clone(), m.clone())))
            .collect()
    };

    // Les nouveaux messages sont lus en parallèle.
    let mut tasks = tokio::task::JoinSet::new();
    for reference in list.messages.iter() {
        if !known.contains_key(&reference.id) {
            tasks.spawn(fetch_message(
                token.to_string(),
                email.to_string(),
                MessageRef {
                    id: reference.id.clone(),
                    thread_id: reference.thread_id.clone(),
                },
            ));
        }
    }
    let mut fetched = HashMap::new();
    while let Some(result) = tasks.join_next().await {
        let message = result
            .map_err(|e| crate::error::AppError::Other(format!("lecture d'un e-mail : {e}")))??;
        fetched.insert(message.id.clone(), message);
    }

    // Garde l'ordre de Gmail (du plus récent au plus ancien).
    let messages: Vec<Message> = list
        .messages
        .iter()
        .filter_map(|r| known.get(&r.id).or_else(|| fetched.get(&r.id)).cloned())
        .collect();

    let mut cache = cache.lock().await;
    cache.messages = messages.iter().map(|m| (m.id.clone(), m.clone())).collect();
    let arrivals = cache.new_arrivals(&messages);
    Ok((
        Inbox {
            email: email.to_string(),
            unread: label.messages_unread,
            messages,
        },
        arrivals,
    ))
}

/// Une notification Windows par nouveau message (au plus trois, puis un résumé).
pub fn notify_arrivals<R: Runtime>(app: &AppHandle<R>, arrivals: &[Message]) {
    const MAX_SEPARATE: usize = 3;
    let mut list: Vec<Notification> = arrivals
        .iter()
        .take(MAX_SEPARATE)
        .map(|m| Notification {
            title: format!("Nouvel e-mail de {}", m.from),
            body: m.subject.clone(),
            sound: Some(Sound::Mail),
        })
        .collect();
    if arrivals.len() > MAX_SEPARATE {
        list.push(Notification {
            title: "Nouveaux e-mails".into(),
            body: format!("Et {} autres messages.", arrivals.len() - MAX_SEPARATE),
            sound: None,
        });
    }
    for notification in list {
        if let Err(e) = notifications::notify(app, &notification) {
            log::warn!("notification de nouvel e-mail impossible : {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(id: &str) -> Message {
        Message {
            id: id.into(),
            from: "Jean".into(),
            subject: "Bonjour".into(),
            snippet: String::new(),
            received_at: 0,
            url: String::new(),
        }
    }

    #[test]
    fn reads_the_sender_name() {
        assert_eq!(
            sender_name("\"Jean Dupont\" <jean@exemple.fr>"),
            "Jean Dupont"
        );
        assert_eq!(sender_name("GitHub <noreply@github.com>"), "GitHub");
        assert_eq!(sender_name("<seul@exemple.fr>"), "seul@exemple.fr");
        assert_eq!(sender_name("brut@exemple.fr"), "brut@exemple.fr");
    }

    #[test]
    fn decodes_html_entities_of_the_snippet() {
        assert_eq!(
            decode_entities("C&#39;est l&amp;heure &quot;ok&quot;"),
            "C'est l&heure \"ok\""
        );
    }

    #[test]
    fn reads_a_message() {
        let meta: MessageMetadata = serde_json::from_str(
            r#"{"id":"m1","snippet":"Votre colis arrive","internalDate":"1764240000000",
               "payload":{"headers":[{"name":"From","value":"La Poste <ne-pas-repondre@laposte.fr>"},
               {"name":"Subject","value":"Livraison"}]}}"#,
        )
        .unwrap();
        let m = to_message("m1".into(), "t1", "moi@gmail.com", meta);
        assert_eq!(m.from, "La Poste");
        assert_eq!(m.subject, "Livraison");
        assert_eq!(m.received_at, 1_764_240_000_000);
        assert_eq!(
            m.url,
            "https://mail.google.com/mail/u/?authuser=moi@gmail.com#inbox/t1"
        );
    }

    #[test]
    fn a_message_without_subject_says_so() {
        let meta: MessageMetadata = serde_json::from_str(r#"{"payload":{"headers":[]}}"#).unwrap();
        assert_eq!(
            to_message("m".into(), "t", "e", meta).subject,
            "(sans objet)"
        );
    }

    #[test]
    fn reads_an_empty_inbox() {
        let list: MessageList = serde_json::from_str(r#"{"resultSizeEstimate":0}"#).unwrap();
        assert!(list.messages.is_empty());
    }

    #[test]
    fn only_messages_arrived_after_the_first_list_are_new() {
        let mut cache = InboxCache::default();
        assert!(cache.new_arrivals(&[message("a"), message("b")]).is_empty());

        let arrivals = cache.new_arrivals(&[message("c"), message("a"), message("b")]);
        assert_eq!(arrivals, vec![message("c")]);

        // Un message déjà signalé ne l'est pas deux fois.
        assert!(cache.new_arrivals(&[message("c")]).is_empty());
    }
}
