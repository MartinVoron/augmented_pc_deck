//! Service commun d'envoi des notifications du système (Windows ou bureau Linux).
//!
//! Les modules (minuteurs, Claude, GitHub, Gmail) passent tous par [`notify`].
//! Un interrupteur global permet de couper toutes les notifications d'un coup.
//! Il est enregistré dans la configuration (`general.notificationsEnabled`).

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_notification::NotificationExt;

use crate::error::{AppError, AppResult};
use crate::modules::settings::ConfigStore;

/// Son joué avec la notification (sons standards du système).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sound {
    /// Son par défaut des notifications.
    Default,
    /// Son de rappel, adapté aux minuteurs.
    Reminder,
    /// Son de nouveau message, adapté aux e-mails.
    Mail,
    /// Alarme, pour ce qui demande une réaction rapide.
    Alarm,
}

impl Sound {
    /// Nom du son attendu par Windows.
    #[cfg_attr(not(windows), allow(dead_code))]
    fn windows_name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Reminder => "Reminder",
            Self::Mail => "Mail",
            Self::Alarm => "Alarm",
        }
    }

    /// Nom du son dans le thème de sons freedesktop (bureaux Linux).
    #[cfg_attr(windows, allow(dead_code))]
    fn freedesktop_name(self) -> &'static str {
        match self {
            Self::Default => "message-new-instant",
            Self::Reminder => "complete",
            Self::Mail => "message-new-email",
            Self::Alarm => "alarm-clock-elapsed",
        }
    }

    /// Nom du son pour le système en cours.
    fn system_name(self) -> &'static str {
        if cfg!(windows) {
            self.windows_name()
        } else {
            self.freedesktop_name()
        }
    }
}

/// Contenu d'une notification.
#[derive(Debug, Clone, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
    /// `None` : notification silencieuse.
    #[serde(default)]
    pub sound: Option<Sound>,
}

/// État partagé : notifications activées ou coupées.
pub struct NotificationSettings {
    enabled: AtomicBool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(true),
        }
    }
}

impl NotificationSettings {
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }
}

/// Vérifie le contenu d'une notification avant l'envoi.
fn validate(notification: &Notification) -> AppResult<()> {
    if notification.title.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "une notification doit avoir un titre".into(),
        ));
    }
    Ok(())
}

/// Envoie une notification, sauf si les notifications sont coupées.
///
/// Renvoie `true` si la notification a été envoyée, `false` si elle a été ignorée.
pub fn notify<R: Runtime>(app: &AppHandle<R>, notification: &Notification) -> AppResult<bool> {
    validate(notification)?;
    if !app.state::<NotificationSettings>().is_enabled() {
        return Ok(false);
    }

    let mut builder = app
        .notification()
        .builder()
        .title(&notification.title)
        .body(&notification.body);
    if let Some(sound) = notification.sound {
        builder = builder.sound(sound.system_name());
    }
    builder
        .show()
        .map_err(|e| AppError::Other(format!("impossible d'afficher la notification : {e}")))?;
    Ok(true)
}

/// Envoie une notification depuis l'interface (par exemple un essai dans les paramètres).
#[tauri::command]
pub fn notifications_send<R: Runtime>(
    app: AppHandle<R>,
    notification: Notification,
) -> AppResult<bool> {
    notify(&app, &notification)
}

/// Indique si les notifications sont activées.
#[tauri::command]
pub fn notifications_is_enabled(settings: State<'_, NotificationSettings>) -> AppResult<bool> {
    Ok(settings.is_enabled())
}

/// Active ou coupe toutes les notifications, et enregistre ce choix.
#[tauri::command]
pub fn notifications_set_enabled(
    settings: State<'_, NotificationSettings>,
    store: State<'_, ConfigStore>,
    enabled: bool,
) -> AppResult<()> {
    store.update(|config| config.general.notifications_enabled = enabled)?;
    settings.set_enabled(enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification(title: &str) -> Notification {
        Notification {
            title: title.into(),
            body: "Le minuteur est terminé".into(),
            sound: None,
        }
    }

    #[test]
    fn each_sound_has_a_name_on_every_system() {
        assert_eq!(Sound::Mail.windows_name(), "Mail");
        assert_eq!(Sound::Mail.freedesktop_name(), "message-new-email");
        let expected = if cfg!(windows) {
            "Alarm"
        } else {
            "alarm-clock-elapsed"
        };
        assert_eq!(Sound::Alarm.system_name(), expected);
    }

    #[test]
    fn enabled_by_default_and_can_be_turned_off() {
        let settings = NotificationSettings::default();
        assert!(settings.is_enabled());
        settings.set_enabled(false);
        assert!(!settings.is_enabled());
    }

    #[test]
    fn rejects_empty_title() {
        let error = validate(&notification("  ")).unwrap_err();
        assert_eq!(error.code(), "invalid_input");
        assert!(validate(&notification("Pomodoro")).is_ok());
    }

    #[test]
    fn reads_notification_from_interface() {
        let json = r#"{ "title": "Gmail", "body": "1 nouveau message", "sound": "mail" }"#;
        let notification: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notification.sound, Some(Sound::Mail));
        assert_eq!(Sound::Mail.windows_name(), "Mail");
    }

    #[test]
    fn sound_is_optional() {
        let json = r#"{ "title": "Claude", "body": "Tâche terminée" }"#;
        let notification: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(notification.sound, None);
    }
}
