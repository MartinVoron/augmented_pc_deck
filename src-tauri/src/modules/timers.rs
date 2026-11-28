//! Minuteurs : comptes à rebours nommés.
//!
//! Les comptes à rebours tournent côté Rust, pas dans l'interface : ils continuent
//! même quand la fenêtre est masquée (où le navigateur ralentit ses minuteries),
//! et la notification part à l'heure.
//!
//! L'interface ne reçoit pas un tic par seconde : elle reçoit l'heure de fin
//! (`endsAt`) et calcule elle-même le temps restant. Chaque changement d'état est
//! envoyé avec l'événement [`CHANGED_EVENT`].

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::error::{AppError, AppResult};
use crate::notifications::{self, Notification, Sound};

/// Événement envoyé à l'interface à chaque changement, avec tous les minuteurs.
pub const CHANGED_EVENT: &str = "timers-changed";

/// Durée maximale d'un compte à rebours : 24 heures.
pub const MAX_DURATION_MS: u64 = 24 * 60 * 60 * 1000;

/// État d'un compte à rebours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Running,
    Paused,
    Finished,
}

/// Un compte à rebours lancé. Un minuteur jamais lancé (ou réinitialisé) n'en a pas.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerRun {
    pub name: String,
    /// Titre et texte de la notification de fin (sinon « Minuteur terminé »).
    #[serde(skip)]
    pub alert: Option<Alert>,
    pub status: Status,
    pub duration_ms: u64,
    /// Temps restant au moment de la pause (0 quand il est terminé).
    pub remaining_ms: u64,
    /// Heure de fin prévue, en millisecondes depuis 1970, quand il tourne.
    pub ends_at: Option<u64>,
    /// Numéro du lancement : une attente plus ancienne ne doit rien terminer.
    #[serde(skip)]
    generation: u64,
}

/// Notification personnalisée envoyée à la fin d'un minuteur (par exemple le Pomodoro).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Alert {
    pub title: String,
    pub body: String,
}

impl TimerRun {
    fn start(name: String, duration_ms: u64, now: u64, generation: u64) -> Self {
        Self {
            name,
            alert: None,
            status: Status::Running,
            duration_ms,
            remaining_ms: duration_ms,
            ends_at: Some(now + duration_ms),
            generation,
        }
    }

    fn pause(&mut self, now: u64) {
        if let (Status::Running, Some(ends_at)) = (self.status, self.ends_at) {
            self.remaining_ms = ends_at.saturating_sub(now);
            self.ends_at = None;
            self.status = Status::Paused;
        }
    }

    fn resume(&mut self, now: u64, generation: u64) {
        if self.status == Status::Paused {
            self.ends_at = Some(now + self.remaining_ms);
            self.status = Status::Running;
            self.generation = generation;
        }
    }

    /// Termine le compte à rebours s'il tourne encore pour ce lancement.
    fn finish(&mut self, generation: u64) -> bool {
        if self.status != Status::Running || self.generation != generation {
            return false;
        }
        self.status = Status::Finished;
        self.remaining_ms = 0;
        self.ends_at = None;
        true
    }
}

/// Tous les comptes à rebours, par clé (`<tuile>:<minuteur>`).
#[derive(Default)]
pub struct Timers {
    runs: Mutex<HashMap<String, TimerRun>>,
    generation: AtomicU64,
}

impl Timers {
    fn next_generation(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn lock(&self) -> AppResult<std::sync::MutexGuard<'_, HashMap<String, TimerRun>>> {
        self.runs
            .lock()
            .map_err(|_| AppError::Other("les minuteurs sont indisponibles".into()))
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// Envoie l'état de tous les minuteurs à l'interface.
fn emit_changed<R: Runtime>(app: &AppHandle<R>, runs: &HashMap<String, TimerRun>) {
    if let Err(e) = app.emit(CHANGED_EVENT, runs) {
        log::warn!("impossible de prévenir l'interface d'un changement de minuteur : {e}");
    }
}

/// Notification envoyée à la fin d'un minuteur.
fn end_notification(run: &TimerRun) -> Notification {
    let (title, body) = match &run.alert {
        Some(alert) => (alert.title.clone(), alert.body.clone()),
        None => (
            "Minuteur terminé".into(),
            format!("« {} » est terminé.", run.name),
        ),
    };
    Notification {
        title,
        body,
        sound: Some(Sound::Reminder),
    }
}

/// Attend la fin d'un lancement, puis termine le minuteur et envoie la notification.
fn schedule<R: Runtime>(app: AppHandle<R>, key: String, generation: u64, delay_ms: u64) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        let timers = app.state::<Timers>();
        let Ok(mut runs) = timers.lock() else { return };
        let Some(run) = runs.get_mut(&key) else {
            return;
        };
        if !run.finish(generation) {
            return;
        }
        let notification = end_notification(run);
        emit_changed(&app, &runs);
        drop(runs);
        if let Err(e) = notifications::notify(&app, &notification) {
            log::warn!("notification de fin de minuteur impossible : {e}");
        }
    });
}

/// Renvoie tous les comptes à rebours lancés.
#[tauri::command]
pub fn timers_list(timers: State<'_, Timers>) -> AppResult<HashMap<String, TimerRun>> {
    Ok(timers.lock()?.clone())
}

/// Démarre un minuteur, ou le reprend s'il est en pause.
/// `alert` remplace la notification de fin par défaut.
#[tauri::command]
pub fn timers_start<R: Runtime>(
    app: AppHandle<R>,
    timers: State<'_, Timers>,
    key: String,
    name: String,
    duration_ms: u64,
    alert: Option<Alert>,
) -> AppResult<TimerRun> {
    if duration_ms == 0 || duration_ms > MAX_DURATION_MS {
        return Err(AppError::InvalidInput(
            "la durée doit être comprise entre 1 seconde et 24 heures".into(),
        ));
    }
    let generation = timers.next_generation();
    let now = now_ms();
    let mut runs = timers.lock()?;
    let run = match runs.get_mut(&key) {
        Some(run) if run.status == Status::Paused => {
            run.name = name;
            run.alert = alert;
            run.resume(now, generation);
            run.clone()
        }
        Some(run) if run.status == Status::Running => return Ok(run.clone()),
        _ => {
            let mut run = TimerRun::start(name, duration_ms, now, generation);
            run.alert = alert;
            runs.insert(key.clone(), run.clone());
            run
        }
    };
    schedule(app.clone(), key, generation, run.remaining_ms);
    emit_changed(&app, &runs);
    Ok(run)
}

/// Met un minuteur en pause.
#[tauri::command]
pub fn timers_pause<R: Runtime>(
    app: AppHandle<R>,
    timers: State<'_, Timers>,
    key: String,
) -> AppResult<()> {
    let mut runs = timers.lock()?;
    let run = runs
        .get_mut(&key)
        .ok_or_else(|| AppError::NotFound("Ce minuteur".into()))?;
    run.pause(now_ms());
    emit_changed(&app, &runs);
    Ok(())
}

/// Remet un minuteur à zéro (il reprend sa durée complète, à l'arrêt).
#[tauri::command]
pub fn timers_reset<R: Runtime>(
    app: AppHandle<R>,
    timers: State<'_, Timers>,
    key: String,
) -> AppResult<()> {
    let mut runs = timers.lock()?;
    if runs.remove(&key).is_some() {
        emit_changed(&app, &runs);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_keeps_the_remaining_time_and_resume_restarts_from_it() {
        let mut run = TimerRun::start("Thé".into(), 60_000, 1_000, 1);
        assert_eq!(run.ends_at, Some(61_000));

        run.pause(21_000);
        assert_eq!(run.status, Status::Paused);
        assert_eq!(run.remaining_ms, 40_000);
        assert_eq!(run.ends_at, None);

        run.resume(100_000, 2);
        assert_eq!(run.status, Status::Running);
        assert_eq!(run.ends_at, Some(140_000));
    }

    #[test]
    fn an_old_wait_does_not_finish_a_resumed_timer() {
        let mut run = TimerRun::start("Pâtes".into(), 10_000, 0, 1);
        run.pause(5_000);
        run.resume(6_000, 2);

        assert!(!run.finish(1));
        assert_eq!(run.status, Status::Running);
        assert!(run.finish(2));
        assert_eq!(run.status, Status::Finished);
        assert_eq!(run.remaining_ms, 0);
    }

    #[test]
    fn a_paused_timer_does_not_finish() {
        let mut run = TimerRun::start("Lessive".into(), 10_000, 0, 1);
        run.pause(2_000);
        assert!(!run.finish(1));
        assert_eq!(run.status, Status::Paused);
    }

    #[test]
    fn end_notification_uses_the_custom_alert() {
        let mut run = TimerRun::start("Thé".into(), 1_000, 0, 1);
        assert_eq!(end_notification(&run).body, "« Thé » est terminé.");
        run.alert = Some(Alert {
            title: "Pomodoro".into(),
            body: "Place à la pause.".into(),
        });
        let notification = end_notification(&run);
        assert_eq!(notification.title, "Pomodoro");
        assert_eq!(notification.sound, Some(Sound::Reminder));
    }

    #[test]
    fn serializes_for_the_interface() {
        let run = TimerRun::start("Thé".into(), 1_000, 0, 7);
        let json = serde_json::to_value(&run).unwrap();
        assert_eq!(json["status"], "running");
        assert_eq!(json["endsAt"], 1_000);
        assert!(json.get("generation").is_none());
    }
}
