//! Linux : choix du serveur d'affichage.
//!
//! Sous Wayland, une application ne peut ni se placer elle-même à l'écran, ni
//! connaître la position de la souris hors de sa fenêtre, et « toujours au premier
//! plan » est souvent ignoré : l'encoche ne pourrait pas se coller au bord ni
//! s'ouvrir au survol (voir `dock`). En session Wayland, l'application passe donc
//! par XWayland (X11), qui permet tout cela. L'utilisateur garde la main : s'il a
//! choisi lui-même `GDK_BACKEND`, rien ne change.

/// Indique s'il faut demander X11 à GTK, d'après les variables d'environnement :
/// session Wayland, XWayland disponible (`DISPLAY`), et `GDK_BACKEND` non choisi.
pub fn should_use_x11(
    wayland_display: Option<&str>,
    session_type: Option<&str>,
    x11_display: Option<&str>,
    gdk_backend: Option<&str>,
) -> bool {
    let set = |value: Option<&str>| value.is_some_and(|v| !v.trim().is_empty());
    let wayland = set(wayland_display) || session_type.is_some_and(|t| t.trim() == "wayland");
    wayland && set(x11_display) && !set(gdk_backend)
}

/// À appeler tout au début, avant que GTK ne démarre. Renvoie `true` si
/// l'application passe par XWayland.
#[cfg(target_os = "linux")]
pub fn prefer_x11() -> bool {
    let var = |name: &str| std::env::var(name).ok();
    let use_x11 = should_use_x11(
        var("WAYLAND_DISPLAY").as_deref(),
        var("XDG_SESSION_TYPE").as_deref(),
        var("DISPLAY").as_deref(),
        var("GDK_BACKEND").as_deref(),
    );
    if use_x11 {
        // Appelé avant tout autre fil d'exécution : la modification est sûre.
        std::env::set_var("GDK_BACKEND", "x11");
    }
    use_x11
}

/// Hors de Linux, rien à choisir.
#[cfg(not(target_os = "linux"))]
pub fn prefer_x11() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_session_goes_through_xwayland() {
        assert!(should_use_x11(Some("wayland-0"), None, Some(":0"), None));
        assert!(should_use_x11(None, Some("wayland"), Some(":1"), Some("")));
    }

    #[test]
    fn user_choice_and_x11_sessions_are_left_alone() {
        // L'utilisateur a choisi le serveur lui-même.
        assert!(!should_use_x11(
            Some("wayland-0"),
            None,
            Some(":0"),
            Some("wayland")
        ));
        // Session X11.
        assert!(!should_use_x11(None, Some("x11"), Some(":0"), None));
        // Pas de XWayland : on reste en Wayland.
        assert!(!should_use_x11(
            Some("wayland-0"),
            Some("wayland"),
            None,
            None
        ));
        // Windows, ou rien de connu.
        assert!(!should_use_x11(None, None, None, None));
    }
}
