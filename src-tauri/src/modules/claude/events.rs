//! Événements de Claude Code, lus dans le JSON envoyé par le hook.
//!
//! Hooks pris en charge (voir la documentation de Claude Code) :
//! - `Notification` : Claude attend une réponse ou une autorisation ;
//! - `Stop` : Claude a fini de répondre.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::notifications::{Notification, Sound};

/// Nombre d'événements gardés en mémoire.
pub const MAX_EVENTS: usize = 50;

/// Ce qui s'est passé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventKind {
    /// Claude attend une réponse ou une autorisation.
    Waiting,
    /// Claude a fini sa tâche.
    Done,
}

/// Un événement, tel qu'affiché dans la tuile.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeEvent {
    pub id: u64,
    pub kind: EventKind,
    /// Nom du dossier du projet (dernier élément de `cwd`).
    pub project: String,
    pub message: String,
    /// Heure de réception, en millisecondes depuis 1970.
    pub received_at: u64,
    pub session_id: String,
}

/// JSON transmis par Claude Code au hook (seuls les champs utiles).
#[derive(Deserialize)]
struct HookInput {
    hook_event_name: String,
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    cwd: String,
    #[serde(default)]
    message: Option<String>,
}

/// Nom du projet : le dernier dossier du chemin (`C:\dev\mon-projet` → `mon-projet`).
fn project_name(cwd: &str) -> String {
    cwd.trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("Claude Code")
        .to_string()
}

/// Lit le JSON d'un hook. `Ok(None)` pour un hook qui ne nous intéresse pas.
pub fn parse(body: &[u8], id: u64, now: u64) -> Result<Option<ClaudeEvent>, serde_json::Error> {
    let input: HookInput = serde_json::from_slice(body)?;
    let (kind, message) = match input.hook_event_name.as_str() {
        "Notification" => (
            EventKind::Waiting,
            input
                .message
                .filter(|m| !m.trim().is_empty())
                .unwrap_or_else(|| "Claude attend votre réponse".into()),
        ),
        "Stop" => (EventKind::Done, "Tâche terminée".into()),
        _ => return Ok(None),
    };
    Ok(Some(ClaudeEvent {
        id,
        kind,
        project: project_name(&input.cwd),
        message,
        received_at: now,
        session_id: input.session_id,
    }))
}

/// Notification Windows d'un événement.
pub fn notification(event: &ClaudeEvent) -> Notification {
    match event.kind {
        EventKind::Waiting => Notification {
            title: format!("Claude attend votre réponse · {}", event.project),
            body: event.message.clone(),
            sound: Some(Sound::Reminder),
        },
        EventKind::Done => Notification {
            title: format!("Claude a terminé · {}", event.project),
            body: "La tâche est finie.".into(),
            sound: Some(Sound::Default),
        },
    }
}

/// Ajoute un événement en tête de liste, en gardant les [`MAX_EVENTS`] plus récents.
pub fn push(events: &mut VecDeque<ClaudeEvent>, event: ClaudeEvent) {
    events.push_front(event);
    events.truncate(MAX_EVENTS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_waiting_notification() {
        let body = br#"{"session_id":"s1","transcript_path":"x","cwd":"C:\\dev\\augmented_pc_deck",
            "hook_event_name":"Notification","message":"Claude needs your permission to use Bash"}"#;
        let event = parse(body, 1, 1000).unwrap().unwrap();
        assert_eq!(event.kind, EventKind::Waiting);
        assert_eq!(event.project, "augmented_pc_deck");
        assert_eq!(event.message, "Claude needs your permission to use Bash");
        assert_eq!(event.session_id, "s1");
    }

    #[test]
    fn reads_a_finished_task() {
        let body =
            br#"{"session_id":"s1","cwd":"/home/moi/projet/","hook_event_name":"Stop","stop_hook_active":false}"#;
        let event = parse(body, 2, 1000).unwrap().unwrap();
        assert_eq!(event.kind, EventKind::Done);
        assert_eq!(event.project, "projet");
        assert_eq!(event.message, "Tâche terminée");
    }

    #[test]
    fn ignores_other_hooks_and_rejects_invalid_json() {
        assert_eq!(
            parse(br#"{"hook_event_name":"PreToolUse"}"#, 1, 0).unwrap(),
            None
        );
        assert!(parse(b"pas du json", 1, 0).is_err());
    }

    #[test]
    fn notification_names_the_project() {
        let body = br#"{"hook_event_name":"Notification","cwd":"C:\\dev\\deck","message":"Autoriser Bash ?"}"#;
        let n = notification(&parse(body, 1, 0).unwrap().unwrap());
        assert_eq!(n.title, "Claude attend votre réponse · deck");
        assert_eq!(n.body, "Autoriser Bash ?");

        let body = br#"{"hook_event_name":"Stop","cwd":"C:\\dev\\deck"}"#;
        let n = notification(&parse(body, 2, 0).unwrap().unwrap());
        assert_eq!(n.title, "Claude a terminé · deck");
    }

    #[test]
    fn project_name_has_a_fallback() {
        assert_eq!(project_name(""), "Claude Code");
        assert_eq!(project_name("C:\\"), "C:");
    }

    #[test]
    fn keeps_only_the_most_recent_events() {
        let mut events = VecDeque::new();
        for id in 0..(MAX_EVENTS as u64 + 5) {
            let body = br#"{"hook_event_name":"Stop"}"#;
            push(&mut events, parse(body, id, 0).unwrap().unwrap());
        }
        assert_eq!(events.len(), MAX_EVENTS);
        assert_eq!(events.front().unwrap().id, MAX_EVENTS as u64 + 4);
    }
}
