//! Linux, Wayland : encoche posée par le protocole « layer shell ».
//!
//! Sous Wayland, une fenêtre ordinaire ne peut pas se placer elle-même : le bureau
//! la met où il veut (au milieu de l'écran sous COSMIC). Les barres et les docks
//! utilisent à la place le protocole « layer shell » (COSMIC, KDE, Sway, Hyprland…),
//! qui colle une fenêtre contre un bord de l'écran. GTK le gère avec la
//! bibliothèque `libgtk-layer-shell`, chargée ici à l'exécution : si elle manque,
//! ou si le bureau ne connaît pas ce protocole (GNOME), rien ne change.
//!
//! Encore expérimental : la page ne s'est pas dessinée dans la fenêtre collée lors
//! des essais sous Sway sans carte graphique. Activé seulement avec la variable
//! d'environnement `AUGMENTED_PC_DECK_LAYER_SHELL=1`.

use std::sync::OnceLock;

use gtk::glib::translate::ToGlibPtr;
use libloading::Library;

use crate::modules::settings::config::DockEdge;

type Window = *mut gtk::ffi::GtkWindow;

/// Fonctions de `libgtk-layer-shell` utilisées.
struct Api {
    _library: Library,
    is_supported: unsafe extern "C" fn() -> i32,
    init_for_window: unsafe extern "C" fn(Window),
    is_layer_window: unsafe extern "C" fn(Window) -> i32,
    set_layer: unsafe extern "C" fn(Window, i32),
    set_anchor: unsafe extern "C" fn(Window, i32, i32),
    set_keyboard_mode: unsafe extern "C" fn(Window, i32),
    set_namespace: unsafe extern "C" fn(Window, *const std::ffi::c_char),
}

// Valeurs des énumérations de gtk-layer-shell.
const LAYER_TOP: i32 = 2;
const EDGE_LEFT: i32 = 0;
const EDGE_RIGHT: i32 = 1;
const EDGE_TOP: i32 = 2;
const EDGE_BOTTOM: i32 = 3;
const KEYBOARD_ON_DEMAND: i32 = 2;

fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(|| {
        // SAFETY : bibliothèque système connue ; les signatures suivent son en-tête
        // gtk-layer-shell.h (version 0.6 et plus).
        unsafe {
            let library = Library::new("libgtk-layer-shell.so.0").ok()?;
            macro_rules! symbol {
                ($name:literal) => {
                    *library.get($name).ok()?
                };
            }
            Some(Api {
                is_supported: symbol!(b"gtk_layer_is_supported\0"),
                init_for_window: symbol!(b"gtk_layer_init_for_window\0"),
                is_layer_window: symbol!(b"gtk_layer_is_layer_window\0"),
                set_layer: symbol!(b"gtk_layer_set_layer\0"),
                set_anchor: symbol!(b"gtk_layer_set_anchor\0"),
                set_keyboard_mode: symbol!(b"gtk_layer_set_keyboard_mode\0"),
                set_namespace: symbol!(b"gtk_layer_set_namespace\0"),
                _library: library,
            })
        }
    })
    .as_ref()
}

/// Indique si `libgtk-layer-shell` est installée (sans démarrer GTK).
pub fn available() -> bool {
    api().is_some()
}

/// Indique si l'encoche peut passer par le layer shell : bibliothèque présente et
/// protocole connu du bureau. À appeler depuis le fil principal, GTK démarré.
pub fn supported() -> bool {
    // SAFETY : fonction sans argument, GTK est initialisé.
    api().is_some_and(|api| unsafe { (api.is_supported)() } != 0)
}

/// Colle la fenêtre au bord `edge` de l'écran, centrée le long de ce bord, au-dessus
/// des autres fenêtres, puis l'affiche. Le protocole ne s'applique qu'à une fenêtre
/// jamais affichée : une fenêtre déjà affichée et pas encore collée renvoie
/// `false` (il faut redémarrer l'application). Une fenêtre déjà collée change
/// seulement de bord. À appeler depuis le fil principal.
pub fn attach(window: &gtk::ApplicationWindow, edge: DockEdge) -> bool {
    use gtk::prelude::{Cast, WidgetExt};
    let Some(api) = api() else {
        return false;
    };
    if !supported() {
        return false;
    }
    let gtk_window: &gtk::Window = window.upcast_ref();
    let raw: Window = gtk_window.to_glib_none().0;
    // SAFETY : `raw` est une fenêtre GTK vivante (empruntée à `window`), et l'on
    // est sur le fil principal.
    unsafe {
        if (api.is_layer_window)(raw) != 0 {
            set_edge(api, raw, edge);
            return true;
        }
        if window.is_realized() {
            return false;
        }
        (api.init_for_window)(raw);
        (api.set_namespace)(raw, c"augmented-pc-deck".as_ptr());
        (api.set_layer)(raw, LAYER_TOP);
        (api.set_keyboard_mode)(raw, KEYBOARD_ON_DEMAND);
        set_edge(api, raw, edge);
    }
    window.show_all();
    true
}

/// Indique si la fenêtre est passée en layer shell.
pub fn is_attached(window: &gtk::ApplicationWindow) -> bool {
    use gtk::prelude::Cast;
    let Some(api) = api() else {
        return false;
    };
    let gtk_window: &gtk::Window = window.upcast_ref();
    let raw: Window = gtk_window.to_glib_none().0;
    // SAFETY : fenêtre GTK vivante, fil principal.
    unsafe { (api.is_layer_window)(raw) != 0 }
}

/// Ancre la fenêtre au seul bord `edge` : le bureau la centre le long de ce bord.
///
/// SAFETY : `raw` doit être une fenêtre en layer shell, sur le fil principal.
unsafe fn set_edge(api: &Api, raw: Window, edge: DockEdge) {
    for (side, anchored) in [
        (EDGE_LEFT, edge == DockEdge::Left),
        (EDGE_RIGHT, edge == DockEdge::Right),
        (EDGE_TOP, edge == DockEdge::Top),
        (EDGE_BOTTOM, edge == DockEdge::Bottom),
    ] {
        (api.set_anchor)(raw, side, i32::from(anchored));
    }
}
