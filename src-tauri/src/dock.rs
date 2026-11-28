//! Ancrage de la fenêtre principale sur un bord de l'écran, en encoche (notch).
//!
//! Repliée, la fenêtre n'est qu'une petite encoche centrée sur le bord choisi,
//! toujours au premier plan. Le Rust suit la souris : quand elle s'arrête sur
//! l'encoche, la fenêtre prend la taille du panneau demandé par l'interface, qui
//! anime l'ouverture ; quand elle sort, l'interface anime la fermeture, puis la
//! fenêtre redevient une encoche. La fenêtre est transparente : seule la forme
//! dessinée par l'interface se voit.
//!
//! L'interface est prévenue de chaque changement par l'événement [`EVENT`].
//!
//! Lâcher la fenêtre normale avec la souris collée à un bord de l'écran l'ancre à
//! ce bord (réglage `window.dockOnDrag`), voir [`on_moved`].

use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Runtime, Size, State,
    WebviewWindow,
};

use crate::error::{AppError, AppResult};
use crate::modules::settings::config::{DockEdge, WindowSettings};
use crate::modules::settings::ConfigStore;
use crate::window::MAIN;

/// Événement envoyé à l'interface à chaque changement (bord, ouverture).
pub const EVENT: &str = "dock-state";

/// Taille de l'encoche en pixels logiques : longueur le long du bord, puis profondeur.
pub const NOTCH_LENGTH: f64 = 180.0;
/// Longueurs permises pour l'encoche repliée, quand l'interface l'élargit pour une
/// activité en direct (morceau en cours, minuteur).
const NOTCH_LENGTH_RANGE: (f64, f64) = (120.0, 480.0);
pub const NOTCH_DEPTH: f64 = 32.0;
/// Taille du panneau ouvert tant que l'interface n'a pas donné la sienne.
const DEFAULT_PANEL: (f64, f64) = (640.0, 360.0);
/// Taille minimale de la fenêtre normale (`minWidth`, `minHeight` de `tauri.conf.json`).
const MIN_WINDOW: (f64, f64) = (480.0, 320.0);
/// Taille de la fenêtre normale quand l'ancienne n'est pas connue.
const NORMAL_WINDOW: (f64, f64) = (1024.0, 640.0);

/// Intervalle de lecture de la position de la souris.
const POLL_MS: u64 = 40;
/// Temps sur l'encoche avant l'ouverture : passer dessus sans s'arrêter ne l'ouvre pas.
pub const OPEN_DELAY_MS: u64 = 80;
/// Temps hors du panneau avant de le replier : un écart de souris ne le ferme pas.
pub const CLOSE_DELAY_MS: u64 = 300;
/// Durée de l'animation de fermeture dans l'interface (`DockedView.svelte`) : la
/// fenêtre ne rapetisse qu'après, pour ne pas couper l'animation.
const CLOSE_ANIMATION_MS: u64 = 300;

/// Distance au bord de l'écran, en pixels physiques, à laquelle lâcher la fenêtre l'ancre.
pub const DRAG_EDGE_MARGIN: f64 = 3.0;
/// Temps sans mouvement après lequel un glisser de fenêtre est considéré fini.
const DRAG_QUIET_MS: u64 = 250;
/// Temps pendant lequel les déplacements faits par l'ancrage lui-même sont ignorés.
const SELF_MOVE_MS: u64 = 800;

/// Rectangle à l'écran, en pixels physiques.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    /// Indique si le point (en pixels physiques) est dans le rectangle.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= f64::from(self.x)
            && y >= f64::from(self.y)
            && x < f64::from(self.x) + f64::from(self.width)
            && y < f64::from(self.y) + f64::from(self.height)
    }
}

/// Place un rectangle de `width` × `height` contre le bord `edge` de la zone de
/// travail `work` (l'écran sans la barre des tâches), centré le long de ce bord.
/// `DockEdge::None` le centre au milieu de l'écran. Trop grand, il est réduit à la
/// taille de la zone.
pub fn fit(work: Rect, edge: DockEdge, width: u32, height: u32) -> Rect {
    let width = width.min(work.width);
    let height = height.min(work.height);
    // Écart entre la zone et le rectangle : toujours positif après la réduction.
    let free_x = i32::try_from(work.width - width).unwrap_or(i32::MAX);
    let free_y = i32::try_from(work.height - height).unwrap_or(i32::MAX);
    let center_x = work.x + free_x / 2;
    let center_y = work.y + free_y / 2;
    let (x, y) = match edge {
        DockEdge::Top => (center_x, work.y),
        DockEdge::Bottom => (center_x, work.y + free_y),
        DockEdge::Left => (work.x, center_y),
        DockEdge::Right => (work.x + free_x, center_y),
        DockEdge::None => (center_x, center_y),
    };
    Rect {
        x,
        y,
        width,
        height,
    }
}

/// Bord de l'écran `monitor` contre lequel se trouve la souris (à `margin` pixels
/// près), ou `DockEdge::None`. Dans un coin, le bord le plus proche l'emporte ; à
/// égalité, le haut ou le bas.
pub fn edge_at(monitor: Rect, x: f64, y: f64, margin: f64) -> DockEdge {
    if !monitor.contains(x, y) {
        return DockEdge::None;
    }
    let right = f64::from(monitor.x) + f64::from(monitor.width) - 1.0;
    let bottom = f64::from(monitor.y) + f64::from(monitor.height) - 1.0;
    let distances = [
        (DockEdge::Top, y - f64::from(monitor.y)),
        (DockEdge::Bottom, bottom - y),
        (DockEdge::Left, x - f64::from(monitor.x)),
        (DockEdge::Right, right - x),
    ];
    distances
        .into_iter()
        .filter(|(_, distance)| *distance <= margin)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(DockEdge::None, |(edge, _)| edge)
}

/// Taille de l'encoche en pixels physiques, pour une longueur `length` en pixels
/// logiques : couchée sur les bords haut et bas, debout sur les bords gauche et droit.
pub fn notch_size(edge: DockEdge, length: f64, scale: f64) -> (u32, u32) {
    let length = to_physical(length, scale);
    let depth = to_physical(NOTCH_DEPTH, scale);
    match edge {
        DockEdge::Left | DockEdge::Right => (depth, length),
        _ => (length, depth),
    }
}

fn to_physical(logical: f64, scale: f64) -> u32 {
    // Valeur bornée et positive : la conversion ne perd rien d'utile.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = (logical.max(1.0) * scale).round().min(f64::from(u32::MAX)) as u32;
    value
}

/// Suit la souris pour décider quand ouvrir et replier le panneau.
#[derive(Debug, Default)]
pub struct Hover {
    /// Panneau ouvert.
    pub expanded: bool,
    inside_ms: u64,
    outside_ms: u64,
}

impl Hover {
    /// Tient compte d'une nouvelle lecture : souris dans la fenêtre ou non, `elapsed_ms`
    /// après la précédente. Renvoie `Some(true)` pour ouvrir, `Some(false)` pour replier.
    pub fn update(&mut self, inside: bool, elapsed_ms: u64) -> Option<bool> {
        if inside {
            self.outside_ms = 0;
            self.inside_ms = self.inside_ms.saturating_add(elapsed_ms);
            if !self.expanded && self.inside_ms >= OPEN_DELAY_MS {
                self.expanded = true;
                return Some(true);
            }
        } else {
            self.inside_ms = 0;
            self.outside_ms = self.outside_ms.saturating_add(elapsed_ms);
            if self.expanded && self.outside_ms >= CLOSE_DELAY_MS {
                self.expanded = false;
                return Some(false);
            }
        }
        None
    }
}

/// État envoyé à l'interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockStatus {
    /// Bord où la fenêtre est ancrée (`none` : fenêtre normale).
    pub edge: DockEdge,
    /// Panneau ouvert (sinon : encoche repliée).
    pub expanded: bool,
}

/// Taille et position de la fenêtre normale, pour y revenir en la détachant.
#[derive(Debug, Clone, Copy)]
struct Restore {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    maximized: bool,
}

#[derive(Debug, Default)]
struct Inner {
    edge: DockEdge,
    expanded: bool,
    /// Taille du panneau ouvert demandée par l'interface, en pixels logiques.
    panel: Option<(f64, f64)>,
    /// Longueur de l'encoche repliée demandée par l'interface, en pixels logiques.
    notch_length: Option<f64>,
    /// Zone de travail de l'écran où la fenêtre est ancrée, et son échelle.
    screen: Option<(Rect, f64)>,
    restore: Option<Restore>,
    /// Change à chaque ancrage ou détachement : arrête le suivi de souris précédent.
    generation: u64,
    /// Réglage `window.dockOnDrag`.
    dock_on_drag: bool,
    /// Glisser de la fenêtre normale en cours.
    drag: Option<Drag>,
    /// Taille et position d'avant le dernier glisser, à retenir si celui-ci ancre la
    /// fenêtre (Windows a pu l'« aimanter » entre-temps).
    drag_restore: Option<Restore>,
    /// Jusqu'à quand ignorer les déplacements faits par l'ancrage lui-même.
    ignore_moves_until: Option<Instant>,
}

/// Glisser de la fenêtre normale en cours.
#[derive(Debug, Clone, Copy)]
struct Drag {
    last_move: Instant,
    start: Option<Restore>,
}

impl Inner {
    fn status(&self) -> DockStatus {
        DockStatus {
            edge: self.edge,
            expanded: self.expanded,
        }
    }

    /// Rectangle de la fenêtre ancrée : encoche ou panneau ouvert.
    fn target(&self) -> Option<Rect> {
        let (work, scale) = self.screen?;
        if self.edge == DockEdge::None {
            return None;
        }
        let (width, height) = if self.expanded {
            let (w, h) = self.panel.unwrap_or(DEFAULT_PANEL);
            (to_physical(w, scale), to_physical(h, scale))
        } else {
            notch_size(self.edge, self.notch_length.unwrap_or(NOTCH_LENGTH), scale)
        };
        Some(fit(work, self.edge, width, height))
    }
}

/// État de l'ancrage, partagé par les commandes et le suivi de la souris.
#[derive(Debug, Default)]
pub struct DockState(Mutex<Inner>);

impl DockState {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Applique le réglage `window.dock`. Renvoie `true` si la fenêtre est ancrée : les
/// autres options d'affichage (premier plan, plein écran) ne s'appliquent alors pas.
///
/// Les appels à la fenêtre sont faits sans tenir le verrou : sous Windows, ils
/// peuvent déclencher tout de suite des événements de fenêtre.
pub fn apply<R: Runtime>(app: &AppHandle<R>, settings: &WindowSettings) -> bool {
    let Some(window) = app.get_webview_window(MAIN) else {
        return false;
    };
    let state = app.state::<DockState>();
    let next = settings.dock;
    let previous = {
        let mut inner = state.lock();
        inner.dock_on_drag = settings.dock_on_drag;
        if inner.edge != next {
            inner.drag = None;
            inner.ignore_moves_until = Some(Instant::now() + Duration::from_millis(SELF_MOVE_MS));
        }
        inner.edge
    };
    if previous == next {
        return next != DockEdge::None;
    }

    if next == DockEdge::None {
        let restore = {
            let mut inner = state.lock();
            inner.edge = DockEdge::None;
            inner.expanded = false;
            inner.generation += 1;
            inner.restore.take()
        };
        let screen = screen_of(&window);
        leave_dock(&window, restore, screen);
        log::info!("fenêtre détachée du bord de l'écran");
        emit(app, state.lock().status());
        return false;
    }

    let generation = if previous == DockEdge::None {
        let captured = capture(&window);
        let screen = screen_of(&window);
        enter_dock(&window);
        let mut inner = state.lock();
        inner.restore = inner.drag_restore.take().or(captured);
        inner.screen = screen;
        inner.generation += 1;
        Some(inner.generation)
    } else {
        None
    };
    let (target, status) = {
        let mut inner = state.lock();
        inner.edge = next;
        inner.expanded = false;
        (inner.target(), inner.status())
    };
    if let Some(rect) = target {
        set_rect(&window, rect);
    }
    if let Some(generation) = generation {
        watch(app.clone(), generation);
    }
    log::info!("fenêtre ancrée au bord {next:?} de l'écran");
    emit(app, status);
    true
}

/// Retient la taille et la position de la fenêtre normale, sans l'agrandissement.
fn current_geometry<R: Runtime>(window: &WebviewWindow<R>) -> Option<Restore> {
    if window.is_maximized().unwrap_or(false) {
        return None;
    }
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    Some(Restore {
        position,
        size,
        maximized: false,
    })
}

/// Retient la taille et la position de la fenêtre normale. Une fenêtre plus petite
/// que la taille minimale (encoche retenue par `window-state` d'une session
/// précédente) n'est pas retenue : elle reviendra au centre, à sa taille par défaut.
fn capture<R: Runtime>(window: &WebviewWindow<R>) -> Option<Restore> {
    let maximized = window.is_maximized().unwrap_or(false);
    if maximized {
        let _ = window.unmaximize();
    }
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let scale = window.scale_factor().unwrap_or(1.0);
    let (min_width, min_height) = (
        to_physical(MIN_WINDOW.0, scale),
        to_physical(MIN_WINDOW.1, scale),
    );
    if size.width < min_width || size.height < min_height {
        return None;
    }
    Some(Restore {
        position,
        size,
        maximized,
    })
}

/// Zone de travail et échelle de l'écran de la fenêtre (à défaut, l'écran principal).
fn screen_of<R: Runtime>(window: &WebviewWindow<R>) -> Option<(Rect, f64)> {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())?;
    let area = monitor.work_area();
    let work = Rect {
        x: area.position.x,
        y: area.position.y,
        width: area.size.width,
        height: area.size.height,
    };
    Some((work, monitor.scale_factor()))
}

/// Prépare la fenêtre à devenir une encoche.
fn enter_dock<R: Runtime>(window: &WebviewWindow<R>) {
    let steps: [(&str, tauri::Result<()>); 6] = [
        ("plein écran", window.set_fullscreen(false)),
        ("taille minimale", window.set_min_size(None::<Size>)),
        ("redimensionnement", window.set_resizable(false)),
        ("premier plan", window.set_always_on_top(true)),
        ("barre des tâches", window.set_skip_taskbar(true)),
        ("ombre", window.set_shadow(false)),
    ];
    log_failures("ancrage", steps);
}

/// Rend à la fenêtre son aspect normal, sa taille et sa position d'avant l'ancrage.
fn leave_dock<R: Runtime>(
    window: &WebviewWindow<R>,
    restore: Option<Restore>,
    screen: Option<(Rect, f64)>,
) {
    let steps: [(&str, tauri::Result<()>); 4] = [
        ("ombre", window.set_shadow(true)),
        ("barre des tâches", window.set_skip_taskbar(false)),
        ("redimensionnement", window.set_resizable(true)),
        (
            "taille minimale",
            window.set_min_size(Some(LogicalSize::new(MIN_WINDOW.0, MIN_WINDOW.1))),
        ),
    ];
    log_failures("détachement", steps);
    match (restore, screen) {
        (Some(restore), _) => {
            let _ = window.set_size(restore.size);
            let _ = window.set_position(restore.position);
            if restore.maximized {
                let _ = window.maximize();
            }
        }
        (None, Some((work, scale))) => {
            let width = to_physical(NORMAL_WINDOW.0, scale);
            let height = to_physical(NORMAL_WINDOW.1, scale);
            set_rect(window, fit(work, DockEdge::None, width, height));
        }
        (None, None) => {
            let _ = window.set_size(LogicalSize::new(NORMAL_WINDOW.0, NORMAL_WINDOW.1));
            let _ = window.center();
        }
    }
}

fn log_failures<const N: usize>(what: &str, steps: [(&str, tauri::Result<()>); N]) {
    for (step, result) in steps {
        if let Err(e) = result {
            log::warn!("{what} de la fenêtre : réglage « {step} » impossible : {e}");
        }
    }
}

fn set_rect<R: Runtime>(window: &WebviewWindow<R>, rect: Rect) {
    if let Err(e) = window.set_position(PhysicalPosition::new(rect.x, rect.y)) {
        log::warn!("impossible de placer la fenêtre ancrée : {e}");
    }
    if let Err(e) = window.set_size(PhysicalSize::new(rect.width, rect.height)) {
        log::warn!("impossible de redimensionner la fenêtre ancrée : {e}");
    }
}

fn window_rect<R: Runtime>(window: &WebviewWindow<R>) -> Option<Rect> {
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    Some(Rect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

fn emit<R: Runtime>(app: &AppHandle<R>, status: DockStatus) {
    if let Err(e) = app.emit(EVENT, status) {
        log::warn!("impossible de prévenir l'interface de l'ancrage : {e}");
    }
}

/// Ouvre ou replie le panneau, si l'ancrage `generation` est toujours en cours.
/// À l'ouverture, la fenêtre grandit tout de suite ; à la fermeture, elle ne
/// rapetisse qu'à la fin de l'animation (voir [`shrink`]).
fn set_expanded<R: Runtime>(app: &AppHandle<R>, generation: u64, expanded: bool) {
    let state = app.state::<DockState>();
    let (target, status) = {
        let mut inner = state.lock();
        if inner.generation != generation || inner.edge == DockEdge::None {
            return;
        }
        inner.expanded = expanded;
        (inner.target(), inner.status())
    };
    if expanded {
        if let (Some(window), Some(rect)) = (app.get_webview_window(MAIN), target) {
            set_rect(&window, rect);
        }
    }
    emit(app, status);
}

/// Remet la fenêtre en encoche si le panneau est toujours replié.
fn shrink<R: Runtime>(app: &AppHandle<R>, generation: u64) {
    let target = {
        let inner = app.state::<DockState>().inner().lock();
        if inner.generation != generation || inner.expanded {
            return;
        }
        inner.target()
    };
    if let (Some(window), Some(rect)) = (app.get_webview_window(MAIN), target) {
        set_rect(&window, rect);
    }
}

/// Suit la souris tant que l'ancrage `generation` dure.
fn watch<R: Runtime>(app: AppHandle<R>, generation: u64) {
    let spawned = thread::Builder::new()
        .name("suivi-encoche".into())
        .spawn(move || {
            let mut hover = Hover::default();
            let mut clock: u64 = 0;
            let mut shrink_at: Option<u64> = None;
            loop {
                thread::sleep(Duration::from_millis(POLL_MS));
                clock += POLL_MS;
                {
                    let inner = app.state::<DockState>().inner().lock();
                    if inner.generation != generation {
                        return;
                    }
                    hover.expanded = inner.expanded;
                }
                let Some(window) = app.get_webview_window(MAIN) else {
                    return;
                };
                if !window.is_visible().unwrap_or(false) {
                    continue;
                }
                // Sous Wayland, la position de la souris n'est pas toujours connue.
                let Ok(cursor) = app.cursor_position() else {
                    continue;
                };
                let inside = window_rect(&window).is_some_and(|r| r.contains(cursor.x, cursor.y));
                match hover.update(inside, POLL_MS) {
                    Some(true) => {
                        shrink_at = None;
                        set_expanded(&app, generation, true);
                    }
                    Some(false) => {
                        shrink_at = Some(clock + CLOSE_ANIMATION_MS);
                        set_expanded(&app, generation, false);
                    }
                    None => {}
                }
                if shrink_at.is_some_and(|at| clock >= at) {
                    shrink_at = None;
                    shrink(&app, generation);
                }
            }
        });
    if let Err(e) = spawned {
        log::error!("impossible de suivre la souris pour l'encoche : {e}");
    }
}

/// La fenêtre principale a bougé. Si c'est l'utilisateur qui la glisse (fenêtre
/// normale, réglage actif), on attend la fin du glisser : bouton de la souris lâché
/// et plus de mouvement. Si la souris est alors collée à un bord de l'écran, la
/// fenêtre s'ancre à ce bord, et le réglage est enregistré.
pub fn on_moved<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(MAIN) else {
        return;
    };
    let now = Instant::now();
    let state = app.state::<DockState>();
    {
        let mut inner = state.lock();
        let ignored = inner.ignore_moves_until.is_some_and(|until| now < until);
        if inner.edge != DockEdge::None || !inner.dock_on_drag || ignored {
            return;
        }
        if let Some(drag) = inner.drag.as_mut() {
            drag.last_move = now;
            return;
        }
    }
    // Premier mouvement d'un glisser : on retient la fenêtre telle qu'elle était.
    let start = current_geometry(&window);
    state.lock().drag = Some(Drag {
        last_move: now,
        start,
    });
    let app = app.clone();
    let spawned = thread::Builder::new()
        .name("glisser-fenetre".into())
        .spawn(move || loop {
            thread::sleep(Duration::from_millis(POLL_MS * 2));
            let state = app.state::<DockState>();
            let drag = {
                let mut inner = state.inner().lock();
                let Some(drag) = inner.drag else {
                    return;
                };
                let quiet = drag.last_move.elapsed() >= Duration::from_millis(DRAG_QUIET_MS);
                if !quiet || mouse_button_down() {
                    continue;
                }
                inner.drag = None;
                drag
            };
            finish_drag(&app, drag);
            return;
        });
    if let Err(e) = spawned {
        log::warn!("impossible de suivre le glisser de la fenêtre : {e}");
        state.lock().drag = None;
    }
}

/// Fin d'un glisser : ancre la fenêtre si la souris est contre un bord de l'écran.
fn finish_drag<R: Runtime>(app: &AppHandle<R>, drag: Drag) {
    let Ok(cursor) = app.cursor_position() else {
        return;
    };
    let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y) else {
        return;
    };
    let bounds = Rect {
        x: monitor.position().x,
        y: monitor.position().y,
        width: monitor.size().width,
        height: monitor.size().height,
    };
    let edge = edge_at(bounds, cursor.x, cursor.y, DRAG_EDGE_MARGIN);
    if edge == DockEdge::None {
        return;
    }
    let store = app.state::<ConfigStore>();
    let mut config = store.get();
    if config.window.dock != DockEdge::None {
        return;
    }
    config.window.dock = edge;
    match store.set(config) {
        Ok(saved) => {
            app.state::<DockState>().lock().drag_restore = drag.start;
            log::info!("fenêtre lâchée contre le bord {edge:?} de l'écran");
            apply(app, &saved.window);
        }
        Err(e) => log::error!("ancrage après un glisser impossible : {e}"),
    }
}

/// Indique si un bouton de la souris est enfoncé (un glisser est encore en cours).
/// Seul Windows le dit ; ailleurs, la fin du mouvement suffit.
#[cfg(windows)]
fn mouse_button_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
    // Le bit de poids fort indique un bouton enfoncé. Boutons physiques : les deux
    // sont vérifiés, pour les souris inversées des gauchers.
    // SAFETY : simple lecture de l'état du clavier et de la souris, sans pointeur.
    let down = |key: u16| unsafe { GetAsyncKeyState(i32::from(key)) } < 0;
    down(VK_LBUTTON.0) || down(VK_RBUTTON.0)
}

#[cfg(not(windows))]
fn mouse_button_down() -> bool {
    false
}

/// État actuel de l'ancrage.
#[tauri::command]
pub fn dock_status(state: State<'_, DockState>) -> DockStatus {
    state.lock().status()
}

/// Taille du panneau ouvert, en pixels logiques, calculée par l'interface d'après
/// les tuiles de la page. Elle est réduite à la taille de l'écran si besoin.
#[tauri::command]
pub fn dock_set_panel<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, DockState>,
    width: f64,
    height: f64,
) -> AppResult<()> {
    if !(width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0) {
        return Err(AppError::Other(format!(
            "taille de panneau invalide : {width} × {height}"
        )));
    }
    let target = {
        let mut inner = state.lock();
        inner.panel = Some((width, height));
        if inner.expanded {
            inner.target()
        } else {
            None
        }
    };
    if let (Some(window), Some(rect)) = (app.get_webview_window(MAIN), target) {
        set_rect(&window, rect);
    }
    Ok(())
}

/// Longueur de l'encoche repliée, en pixels logiques : l'interface l'allonge quand
/// elle montre une activité en direct, et la remet à `NOTCH_LENGTH` ensuite.
#[tauri::command]
pub fn dock_set_notch<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, DockState>,
    length: f64,
) -> AppResult<()> {
    if !length.is_finite() {
        return Err(AppError::Other(format!(
            "longueur d'encoche invalide : {length}"
        )));
    }
    let length = length.clamp(NOTCH_LENGTH_RANGE.0, NOTCH_LENGTH_RANGE.1);
    let target = {
        let mut inner = state.lock();
        inner.notch_length = Some(length);
        if inner.expanded {
            None
        } else {
            inner.target()
        }
    };
    if let (Some(window), Some(rect)) = (app.get_webview_window(MAIN), target) {
        set_rect(&window, rect);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
    };

    #[test]
    fn notch_is_centered_on_each_edge() {
        assert_eq!(
            fit(SCREEN, DockEdge::Top, 180, 32),
            Rect {
                x: 870,
                y: 0,
                width: 180,
                height: 32
            }
        );
        assert_eq!(fit(SCREEN, DockEdge::Bottom, 180, 32).y, 1008);
        assert_eq!(
            fit(SCREEN, DockEdge::Left, 32, 180),
            Rect {
                x: 0,
                y: 430,
                width: 32,
                height: 180
            }
        );
        assert_eq!(fit(SCREEN, DockEdge::Right, 32, 180).x, 1888);
        let center = fit(SCREEN, DockEdge::None, 1024, 640);
        assert_eq!((center.x, center.y), (448, 200));
    }

    #[test]
    fn panel_follows_the_screen_it_is_docked_on() {
        // Second écran à droite du premier, barre des tâches de 40 px en haut.
        let second = Rect {
            x: 1920,
            y: 40,
            width: 2560,
            height: 1400,
        };
        assert_eq!(
            fit(second, DockEdge::Top, 800, 300),
            Rect {
                x: 2800,
                y: 40,
                width: 800,
                height: 300
            }
        );
        assert_eq!(fit(second, DockEdge::Bottom, 800, 300).y, 1140);
        // Écran à gauche, en coordonnées négatives.
        let left = Rect {
            x: -1280,
            y: 0,
            width: 1280,
            height: 1024,
        };
        assert_eq!(fit(left, DockEdge::Right, 300, 600).x, -300);
    }

    #[test]
    fn panel_too_big_is_reduced_to_the_screen() {
        let rect = fit(SCREEN, DockEdge::Left, 3000, 2000);
        assert_eq!(
            rect,
            Rect {
                x: 0,
                y: 0,
                width: 1920,
                height: 1040
            }
        );
    }

    #[test]
    fn notch_size_follows_the_edge_and_the_scale() {
        assert_eq!(notch_size(DockEdge::Top, NOTCH_LENGTH, 1.0), (180, 32));
        assert_eq!(notch_size(DockEdge::Bottom, NOTCH_LENGTH, 1.5), (270, 48));
        assert_eq!(notch_size(DockEdge::Right, NOTCH_LENGTH, 1.25), (40, 225));
        // Encoche allongée pour une activité en direct.
        assert_eq!(notch_size(DockEdge::Top, 300.0, 1.0), (300, 32));
    }

    #[test]
    fn rect_contains_its_inside_only() {
        let notch = fit(SCREEN, DockEdge::Top, 180, 32);
        assert!(notch.contains(870.0, 0.0));
        assert!(notch.contains(1049.5, 31.5));
        assert!(!notch.contains(1050.0, 10.0));
        assert!(!notch.contains(900.0, 32.0));
        assert!(!notch.contains(869.0, 5.0));
    }

    #[test]
    fn hover_opens_after_a_short_stop_and_closes_after_leaving() {
        let mut hover = Hover::default();
        // Passer sur l'encoche sans s'arrêter ne l'ouvre pas.
        assert_eq!(hover.update(true, 40), None);
        assert_eq!(hover.update(false, 40), None);
        assert_eq!(hover.update(true, 40), None);
        assert_eq!(hover.update(true, 40), Some(true));
        assert!(hover.expanded);
        // Rester dedans ne renvoie plus rien.
        assert_eq!(hover.update(true, 40), None);
        // Un court écart ne ferme pas ; une vraie sortie, si.
        for _ in 0..5 {
            assert_eq!(hover.update(false, 40), None);
        }
        assert_eq!(hover.update(true, 40), None);
        let mut closed = None;
        for _ in 0..(CLOSE_DELAY_MS / 40 + 1) {
            if let Some(change) = hover.update(false, 40) {
                closed = Some(change);
            }
        }
        assert_eq!(closed, Some(false));
        assert!(!hover.expanded);
    }

    #[test]
    fn dropping_against_an_edge_finds_that_edge() {
        let m = DRAG_EDGE_MARGIN;
        assert_eq!(edge_at(SCREEN, 960.0, 0.0, m), DockEdge::Top);
        assert_eq!(edge_at(SCREEN, 960.0, 2.0, m), DockEdge::Top);
        assert_eq!(edge_at(SCREEN, 960.0, 1039.0, m), DockEdge::Bottom);
        assert_eq!(edge_at(SCREEN, 0.0, 500.0, m), DockEdge::Left);
        assert_eq!(edge_at(SCREEN, 1919.0, 500.0, m), DockEdge::Right);
        // Loin des bords, ou hors de l'écran : rien.
        assert_eq!(edge_at(SCREEN, 960.0, 20.0, m), DockEdge::None);
        assert_eq!(edge_at(SCREEN, 1920.0, 500.0, m), DockEdge::None);
    }

    #[test]
    fn corners_go_to_the_nearest_edge() {
        let m = DRAG_EDGE_MARGIN;
        assert_eq!(edge_at(SCREEN, 0.0, 0.0, m), DockEdge::Top);
        assert_eq!(edge_at(SCREEN, 0.0, 2.0, m), DockEdge::Left);
        assert_eq!(edge_at(SCREEN, 1918.0, 1039.0, m), DockEdge::Bottom);
    }

    #[test]
    fn edges_of_a_second_screen_are_found() {
        let second = Rect {
            x: -1280,
            y: -200,
            width: 1280,
            height: 1024,
        };
        let m = DRAG_EDGE_MARGIN;
        assert_eq!(edge_at(second, -640.0, -200.0, m), DockEdge::Top);
        assert_eq!(edge_at(second, -1.0, 300.0, m), DockEdge::Right);
        assert_eq!(edge_at(second, -1280.0, 300.0, m), DockEdge::Left);
    }

    #[test]
    fn status_is_sent_in_camel_case() {
        let status = DockStatus {
            edge: DockEdge::Bottom,
            expanded: true,
        };
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            r#"{"edge":"bottom","expanded":true}"#
        );
    }

    #[test]
    fn docked_window_targets_the_notch_then_the_panel() {
        let mut inner = Inner {
            edge: DockEdge::Top,
            screen: Some((SCREEN, 1.0)),
            ..Inner::default()
        };
        assert_eq!(inner.target(), Some(fit(SCREEN, DockEdge::Top, 180, 32)));
        inner.notch_length = Some(300.0);
        assert_eq!(inner.target(), Some(fit(SCREEN, DockEdge::Top, 300, 32)));
        inner.expanded = true;
        assert_eq!(inner.target(), Some(fit(SCREEN, DockEdge::Top, 640, 360)));
        inner.panel = Some((900.0, 320.0));
        assert_eq!(inner.target(), Some(fit(SCREEN, DockEdge::Top, 900, 320)));
        inner.edge = DockEdge::None;
        assert_eq!(inner.target(), None);
    }
}
