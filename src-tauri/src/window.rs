//! Fenêtre principale : position retenue, options d'affichage, démarrage avec la session
//! et icône dans la zone de notification.

use tauri::menu::{Menu, MenuItem};
use tauri::plugin::TauriPlugin;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_window_state::StateFlags;

use crate::error::{AppError, AppResult};
use crate::modules::settings::config::WindowSettings;

/// Nom de la fenêtre principale (voir `tauri.conf.json`).
pub(crate) const MAIN: &str = "main";

/// Retient la taille, la position (donc l'écran) et l'agrandissement de la fenêtre.
/// Le plein écran n'en fait pas partie : il vient de la configuration.
pub fn state_plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_window_state::Builder::new()
        .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
        .build()
}

/// Lance l'application à l'ouverture de la session (activé depuis les paramètres) :
/// registre sous Windows, fichier `.desktop` dans `~/.config/autostart` sous Linux.
pub fn autostart_plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_autostart::Builder::new().build()
}

/// Applique les options d'affichage de la configuration à la fenêtre principale.
pub fn apply<R: Runtime>(app: &AppHandle<R>, settings: &WindowSettings) {
    let Some(window) = app.get_webview_window(MAIN) else {
        return;
    };
    if let Err(e) = window.set_always_on_top(settings.always_on_top) {
        log::warn!("impossible de régler « toujours au premier plan » : {e}");
    }
    if let Err(e) = window.set_fullscreen(settings.fullscreen) {
        log::warn!("impossible de régler le plein écran : {e}");
    }
}

/// Fermer la fenêtre principale (rond rouge, Alt+F4) la masque dans la zone de
/// notification au lieu de quitter : les minuteurs et notifications continuent.
/// Pour quitter, utiliser « Quitter » dans le menu de l'icône.
///
/// Déplacer la fenêtre normale peut l'ancrer à un bord de l'écran (voir `dock`).
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if let WindowEvent::Moved(_) = event {
        if window.label() == MAIN {
            crate::dock::on_moved(window.app_handle());
        }
    }
    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == MAIN {
            api.prevent_close();
            if let Err(e) = window.hide() {
                log::warn!("impossible de masquer la fenêtre : {e}");
            }
        }
    }
}

/// Affiche la fenêtre principale et lui donne le focus.
fn show<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.hide();
    }
}

/// Icône dans la zone de notification, avec le menu Afficher, Masquer et Quitter.
/// Sous Windows, un clic gauche sur l'icône affiche la fenêtre. Sous Linux
/// (AppIndicator), l'icône ne reçoit pas les clics : tout clic ouvre le menu, qui
/// commence donc par « Afficher ».
pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "Afficher", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Masquer", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Augmented PC Deck")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show(app),
            "hide" => hide(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn autostart_error(error: impl std::fmt::Display) -> AppError {
    AppError::Other(format!(
        "réglage du lancement au démarrage impossible : {error}"
    ))
}

/// Indique si l'application se lance à l'ouverture de la session.
#[tauri::command]
pub fn window_autostart_get<R: Runtime>(app: AppHandle<R>) -> AppResult<bool> {
    app.autolaunch().is_enabled().map_err(autostart_error)
}

/// Active ou désactive le lancement à l'ouverture de la session.
#[tauri::command]
pub fn window_autostart_set<R: Runtime>(app: AppHandle<R>, enabled: bool) -> AppResult<()> {
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch.enable().map_err(autostart_error)?;
    } else {
        autolaunch.disable().map_err(autostart_error)?;
    }
    log::info!(
        "lancement au démarrage : {}",
        if enabled { "activé" } else { "désactivé" }
    );
    Ok(())
}
