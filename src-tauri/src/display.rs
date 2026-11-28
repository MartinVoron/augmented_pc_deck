//! Linux : choix du serveur d'affichage.
//!
//! Sous Wayland, une fenêtre ordinaire ne peut ni se placer elle-même à l'écran,
//! ni connaître la position de la souris hors d'elle. Deux façons d'avoir quand
//! même l'encoche :
//! - le protocole « layer shell » (COSMIC, KDE, Sway, Hyprland…), qui colle une
//!   fenêtre contre un bord de l'écran : l'application reste alors en Wayland
//!   (voir `layer`, encore expérimental) ;
//! - sinon (GNOME n'a pas ce protocole), passer par XWayland (X11), où la fenêtre
//!   peut se placer elle-même.
//!
//! L'utilisateur garde la main : s'il a choisi lui-même `GDK_BACKEND`, rien ne change.

use std::sync::atomic::{AtomicBool, Ordering};

/// Serveur d'affichage retenu au démarrage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// Choix de GTK ou de l'utilisateur (Windows, session X11, `GDK_BACKEND` fixé).
    Default,
    /// Session Wayland, encoche par le layer shell.
    Wayland,
    /// Session Wayland, encoche par XWayland.
    Xwayland,
}

/// Environnement lu au démarrage.
#[derive(Debug, Default, Clone, Copy)]
pub struct Env<'a> {
    pub wayland_display: Option<&'a str>,
    pub session_type: Option<&'a str>,
    pub x11_display: Option<&'a str>,
    pub gdk_backend: Option<&'a str>,
    /// `XDG_CURRENT_DESKTOP`, par exemple `COSMIC` ou `ubuntu:GNOME`.
    pub desktop: Option<&'a str>,
    /// `libgtk-layer-shell` est installée et le layer shell demandé
    /// (`AUGMENTED_PC_DECK_LAYER_SHELL=1`, encore expérimental).
    pub layer_shell: bool,
}

/// Bureaux sans layer shell : ceux bâtis sur Mutter (GNOME).
const NO_LAYER_SHELL: [&str; 4] = ["gnome", "unity", "pantheon", "budgie"];

/// Choisit le serveur d'affichage d'après l'environnement.
pub fn choose(env: Env<'_>) -> Backend {
    let set = |value: Option<&str>| value.is_some_and(|v| !v.trim().is_empty());
    let wayland =
        set(env.wayland_display) || env.session_type.is_some_and(|t| t.trim() == "wayland");
    if !wayland {
        return Backend::Default;
    }
    let desktop = env.desktop.unwrap_or_default().to_lowercase();
    let mutter = desktop
        .split(':')
        .any(|name| NO_LAYER_SHELL.contains(&name.trim()));
    // Le layer shell demandé passe avant `GDK_BACKEND` (que certains bureaux,
    // comme COSMIC, fixent eux-mêmes).
    if env.layer_shell && !mutter {
        return Backend::Wayland;
    }
    // Un seul serveur choisi : on le garde. Une liste (« wayland,x11 ») n'est
    // qu'un ordre de préférence par défaut.
    let chosen = env
        .gdk_backend
        .is_some_and(|v| !v.trim().is_empty() && !v.contains(','));
    if chosen {
        Backend::Default
    } else if set(env.x11_display) {
        Backend::Xwayland
    } else {
        Backend::Default
    }
}

static NATIVE_WAYLAND: AtomicBool = AtomicBool::new(false);
static WAYLAND_SESSION: AtomicBool = AtomicBool::new(false);
static SUMMARY: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Indique si la session est Wayland, quel que soit le serveur retenu : la
/// position de la souris et des fenêtres n'y est pas fiable (souvent 0,0).
pub fn wayland_session() -> bool {
    WAYLAND_SESSION.load(Ordering::Relaxed)
}

/// Résumé de l'environnement lu au démarrage, pour le journal.
pub fn summary() -> &'static str {
    SUMMARY.get().map_or("", String::as_str)
}

/// Indique si l'application tourne en Wayland avec le layer shell pour l'encoche.
pub fn native_wayland() -> bool {
    NATIVE_WAYLAND.load(Ordering::Relaxed)
}

/// À appeler tout au début, avant que GTK ne démarre.
#[cfg(target_os = "linux")]
pub fn setup() -> Backend {
    let var = |name: &str| std::env::var(name).ok();
    let (wayland_display, session_type, x11_display, gdk_backend, desktop) = (
        var("WAYLAND_DISPLAY"),
        var("XDG_SESSION_TYPE"),
        var("DISPLAY"),
        var("GDK_BACKEND"),
        var("XDG_CURRENT_DESKTOP"),
    );
    let wanted = var("AUGMENTED_PC_DECK_LAYER_SHELL").is_some_and(|v| v.trim() == "1");
    let library = crate::layer::available();
    let env = Env {
        wayland_display: wayland_display.as_deref(),
        session_type: session_type.as_deref(),
        x11_display: x11_display.as_deref(),
        gdk_backend: gdk_backend.as_deref(),
        desktop: desktop.as_deref(),
        layer_shell: wanted && library,
    };
    let backend = choose(env);
    let wayland = choose(Env {
        gdk_backend: None,
        x11_display: Some(":0"),
        layer_shell: false,
        ..env
    }) != Backend::Default;
    WAYLAND_SESSION.store(wayland, Ordering::Relaxed);
    let _ = SUMMARY.set(format!(
        "session {}, bureau {}, GDK_BACKEND {}, layer shell demandé : {}, \
         libgtk-layer-shell : {}",
        session_type.as_deref().unwrap_or("?"),
        desktop.as_deref().unwrap_or("?"),
        gdk_backend.as_deref().unwrap_or("non fixé"),
        if wanted { "oui" } else { "non" },
        if library { "trouvée" } else { "absente" },
    ));
    match backend {
        Backend::Wayland => {
            // Appelé avant tout autre fil d'exécution : la modification est sûre.
            std::env::set_var("GDK_BACKEND", "wayland");
            NATIVE_WAYLAND.store(true, Ordering::Relaxed);
        }
        Backend::Xwayland => std::env::set_var("GDK_BACKEND", "x11"),
        Backend::Default => {}
    }
    backend
}

/// Hors de Linux, rien à choisir.
#[cfg(not(target_os = "linux"))]
pub fn setup() -> Backend {
    Backend::Default
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wayland<'a>(desktop: &'a str, layer_shell: bool) -> Env<'a> {
        Env {
            wayland_display: Some("wayland-0"),
            session_type: Some("wayland"),
            x11_display: Some(":0"),
            desktop: Some(desktop),
            layer_shell,
            ..Env::default()
        }
    }

    #[test]
    fn cosmic_and_kde_use_the_layer_shell() {
        assert_eq!(choose(wayland("COSMIC", true)), Backend::Wayland);
        assert_eq!(choose(wayland("KDE", true)), Backend::Wayland);
        assert_eq!(choose(wayland("sway", true)), Backend::Wayland);
    }

    #[test]
    fn gnome_or_missing_library_goes_through_xwayland() {
        assert_eq!(choose(wayland("ubuntu:GNOME", true)), Backend::Xwayland);
        assert_eq!(choose(wayland("pop:GNOME", true)), Backend::Xwayland);
        assert_eq!(choose(wayland("COSMIC", false)), Backend::Xwayland);
        let no_xwayland = Env {
            x11_display: None,
            ..wayland("COSMIC", false)
        };
        assert_eq!(choose(no_xwayland), Backend::Default);
    }

    #[test]
    fn requested_layer_shell_wins_over_gdk_backend() {
        let cosmic = Env {
            gdk_backend: Some("wayland"),
            ..wayland("COSMIC", true)
        };
        assert_eq!(choose(cosmic), Backend::Wayland);
        // Une liste de préférence n'est pas un choix.
        let list = Env {
            gdk_backend: Some("wayland,x11"),
            ..wayland("COSMIC", false)
        };
        assert_eq!(choose(list), Backend::Xwayland);
    }

    #[test]
    fn user_choice_and_x11_sessions_are_left_alone() {
        let chosen = Env {
            gdk_backend: Some("x11"),
            ..wayland("COSMIC", false)
        };
        assert_eq!(choose(chosen), Backend::Default);
        let x11 = Env {
            session_type: Some("x11"),
            x11_display: Some(":0"),
            layer_shell: true,
            ..Env::default()
        };
        assert_eq!(choose(x11), Backend::Default);
        assert_eq!(choose(Env::default()), Backend::Default);
    }
}
