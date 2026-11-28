//! Ancrage de la fenêtre principale sur un bord de l'écran, en encoche (notch).
//!
//! La fenêtre ancrée garde toujours la taille du panneau ouvert, contre le bord
//! choisi et centrée le long de ce bord, au premier plan. Elle est transparente :
//! seule la forme dessinée par l'interface se voit. Repliée, l'interface n'y dessine
//! qu'une petite encoche, et la fenêtre laisse passer les clics vers ce qui est
//! dessous. Le Rust suit la souris : quand elle s'arrête sur l'encoche, le panneau
//! s'ouvre (l'interface anime l'encoche jusqu'à lui) et la fenêtre reçoit les clics ;
//! quand elle sort du panneau, il se replie. La fenêtre n'est jamais redimensionnée
//! pendant les animations : pas de saut ni de clignotement.
//!
//! Sous Linux, la position de la souris n'est pas fiable (XWayland ne la donne
//! que quand elle est sur une fenêtre de l'application) et le bureau peut refuser
//! de placer la fenêtre. La fenêtre y garde donc la taille de l'encoche quand elle
//! est repliée (sans laisser passer les clics), et c'est l'interface qui dit quand
//! la souris entre dans la fenêtre ou en sort (commande [`dock_pointer`]).
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

/// Fenêtre repliée de la taille du panneau, qui laisse passer les clics, et souris
/// suivie par le Rust. Sinon (Linux) : fenêtre de la taille de l'encoche, et souris
/// suivie par l'interface.
const CLICK_THROUGH: bool = cfg!(not(target_os = "linux"));
/// Sans `CLICK_THROUGH`, durée de l'animation de fermeture dans l'interface : la
/// fenêtre ne rapetisse qu'après, pour ne pas couper l'animation.
const CLOSE_ANIMATION_MS: u64 = 300;

/// Distance au bord de l'écran, en pixels physiques, à laquelle lâcher la fenêtre l'ancre.
pub const DRAG_EDGE_MARGIN: f64 = 3.0;
/// Temps sans mouvement après lequel un glisser de fenêtre est considéré fini.
const DRAG_QUIET_MS: u64 = 250;
/// Écart permis entre la souris et la fenêtre à la fin d'un glisser, en pixels
/// physiques (le bureau a pu aimanter la fenêtre).
const DRAG_WINDOW_MARGIN: u32 = 24;
/// Temps après le démarrage pendant lequel les déplacements sont ceux du bureau.
const STARTUP_GRACE: Duration = Duration::from_secs(2);
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

    /// Indique si `other` tient entièrement dans le rectangle.
    pub fn holds(&self, other: Rect) -> bool {
        let end = |start: i32, size: u32| i64::from(start) + i64::from(size);
        other.x >= self.x
            && other.y >= self.y
            && end(other.x, other.width) <= end(self.x, self.width)
            && end(other.y, other.height) <= end(self.y, self.height)
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

/// Vérifie que le bureau a bien placé la fenêtre ancrée là où elle doit être :
/// certains (COSMIC, Weston) placent eux-mêmes les fenêtres à leur ouverture.
#[derive(Debug, Default)]
pub struct Placement {
    misplaced_ms: u64,
    tries: u32,
    refused: bool,
}

/// Temps pendant lequel la fenêtre peut être mal placée avant d'être replacée.
const PLACEMENT_GRACE_MS: u64 = 400;
/// Nombre de tentatives pour replacer la fenêtre avant d'abandonner.
const PLACEMENT_TRIES: u32 = 5;

/// Ce qu'il faut faire de la fenêtre ancrée, d'après sa position réelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placed {
    /// Bien placée, ou mal placée depuis trop peu de temps pour s'inquiéter.
    Fine,
    /// Mal placée : la replacer.
    Retry,
    /// Toujours mal placée après plusieurs essais : le bureau refuse.
    Refused,
}

impl Placement {
    /// Tient compte de la position réelle `actual` de la fenêtre et de celle
    /// voulue `target`, `elapsed_ms` après la lecture précédente.
    pub fn check(&mut self, actual: Rect, target: Rect, elapsed_ms: u64) -> Placed {
        if self.refused {
            return Placed::Fine;
        }
        if (actual.x, actual.y) == (target.x, target.y) {
            self.misplaced_ms = 0;
            self.tries = 0;
            return Placed::Fine;
        }
        self.misplaced_ms = self.misplaced_ms.saturating_add(elapsed_ms);
        if self.misplaced_ms < PLACEMENT_GRACE_MS {
            return Placed::Fine;
        }
        self.misplaced_ms = 0;
        if self.tries < PLACEMENT_TRIES {
            self.tries += 1;
            return Placed::Retry;
        }
        self.refused = true;
        log::warn!(
            "le bureau ne laisse pas placer l'encoche contre le bord de l'écran \
             (voulue en {},{}, placée en {},{}) : retour à la fenêtre normale",
            target.x,
            target.y,
            actual.x,
            actual.y
        );
        Placed::Refused
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
    /// Le bureau a refusé de placer l'encoche contre le bord : la fenêtre est
    /// revenue en fenêtre normale.
    pub refused: bool,
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
    /// Position de la fenêtre normale à la fin du dernier glisser (ou au détachement) :
    /// le point de départ du glisser suivant. Certains systèmes ne signalent le
    /// déplacement qu'une fois la fenêtre lâchée.
    settled: Option<PhysicalPosition<i32>>,
    /// Jusqu'à quand ignorer les déplacements faits par l'ancrage lui-même.
    ignore_moves_until: Option<Instant>,
    /// Souris dans la fenêtre, d'après l'interface (sans `CLICK_THROUGH`).
    pointer_inside: bool,
    /// Premier réglage appliqué (démarrage de l'application).
    started: Option<Instant>,
    /// Le bureau a refusé de placer l'encoche (voir [`Placement`]).
    refused: bool,
    /// La fenêtre principale vient d'être recréée et n'est pas encore affichée.
    fresh_window: bool,
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
            refused: self.refused,
        }
    }

    /// Rectangle de la fenêtre ancrée : celui du panneau ouvert, ou celui de
    /// l'encoche quand elle est repliée et ne laisse pas passer les clics.
    fn target(&self, click_through: bool) -> Option<Rect> {
        let (work, scale) = self.screen?;
        if self.edge == DockEdge::None {
            return None;
        }
        let (width, height) = if self.expanded || click_through {
            let (w, h) = self.panel.unwrap_or(DEFAULT_PANEL);
            (to_physical(w, scale), to_physical(h, scale))
        } else {
            self.notch(scale)
        };
        Some(fit(work, self.edge, width, height))
    }

    /// Taille de l'encoche repliée, en pixels physiques.
    fn notch(&self, scale: f64) -> (u32, u32) {
        notch_size(self.edge, self.notch_length.unwrap_or(NOTCH_LENGTH), scale)
    }

    /// Zone où la souris ouvre le panneau ou le garde ouvert, dans la fenêtre
    /// `window` telle qu'elle est vraiment à l'écran (le bureau a pu la placer
    /// ailleurs) : l'encoche, dessinée par l'interface contre le bord et centrée
    /// le long de celui-ci, ou toute la fenêtre quand le panneau est ouvert.
    fn hover_rect(&self, window: Rect) -> Option<Rect> {
        if self.edge == DockEdge::None {
            return None;
        }
        if self.expanded {
            return Some(window);
        }
        let scale = self.screen.map_or(1.0, |(_, scale)| scale);
        let (width, height) = self.notch(scale);
        Some(fit(window, self.edge, width, height))
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
    let (previous, fresh) = {
        let mut inner = state.lock();
        // Fenêtre jamais affichée : au démarrage, ou recréée (voir `recreate_window`).
        let fresh = inner.started.is_none() || std::mem::take(&mut inner.fresh_window);
        inner.started.get_or_insert_with(Instant::now);
        inner.dock_on_drag = settings.dock_on_drag;
        if inner.edge != next {
            inner.drag = None;
            inner.ignore_moves_until = Some(Instant::now() + Duration::from_millis(SELF_MOVE_MS));
        }
        (inner.edge, fresh)
    };
    if previous == next {
        if fresh && next == DockEdge::None {
            let _ = window.show();
        }
        return next != DockEdge::None;
    }

    // Sous Wayland, seule une fenêtre jamais affichée peut se coller au bord, et
    // une fenêtre collée ne redevient pas ordinaire : pour ancrer ou détacher en
    // cours de route, la fenêtre est remplacée par une neuve.
    let crosses = (previous == DockEdge::None) != (next == DockEdge::None);
    if crosses && crate::display::native_wayland() && !fresh {
        recreate_window(app, settings.clone());
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
        let restore = restore.map(|r| keep_visible(&window, r, screen));
        let position = leave_dock(&window, restore, screen);
        state.lock().settled = position;
        if fresh {
            let _ = window.show();
        }
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
        inner.pointer_inside = false;
        inner.generation += 1;
        Some(inner.generation)
    } else {
        None
    };
    let (target, status) = {
        let mut inner = state.lock();
        inner.edge = next;
        inner.expanded = false;
        inner.refused = false;
        (inner.target(CLICK_THROUGH), inner.status())
    };
    attach_layer(&window, next);
    if let Some(rect) = target {
        set_rect(&window, rect);
    }
    pass_clicks(&window, CLICK_THROUGH);
    if let Some(generation) = generation {
        watch(app.clone(), generation);
    }
    log::info!("fenêtre ancrée au bord {next:?} de l'écran");
    emit(app, status);
    true
}

/// Remplace la fenêtre principale par une neuve, masquée, puis applique `settings`
/// à celle-ci (sous Wayland, pour l'ancrer par le layer shell ou la détacher).
/// Fait dans un fil à part : la fermeture de l'ancienne fenêtre passe par le fil
/// principal, qu'il ne faut pas bloquer.
fn recreate_window<R: Runtime>(app: &AppHandle<R>, settings: WindowSettings) {
    let app = app.clone();
    let spawned = thread::Builder::new()
        .name("nouvelle-fenetre".into())
        .spawn(move || {
            let Some(config) = app
                .config()
                .app
                .windows
                .iter()
                .find(|w| w.label == MAIN)
                .cloned()
            else {
                log::error!("configuration de la fenêtre principale introuvable");
                return;
            };
            if let Some(old) = app.get_webview_window(MAIN) {
                let _ = old.destroy();
            }
            // Le nom de la fenêtre n'est libre qu'une fois l'ancienne fermée.
            for _ in 0..50 {
                if app.get_webview_window(MAIN).is_none() {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
            match tauri::WebviewWindowBuilder::from_config(&app, &config)
                .and_then(tauri::WebviewWindowBuilder::build)
            {
                Ok(_) => {
                    log::info!("fenêtre principale recréée");
                    app.state::<DockState>().lock().fresh_window = true;
                    apply(&app, &settings);
                }
                Err(e) => log::error!("impossible de recréer la fenêtre principale : {e}"),
            }
        });
    if let Err(e) = spawned {
        log::error!("impossible de recréer la fenêtre principale : {e}");
    }
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

/// `rect` agrandi de `margin` pixels de chaque côté.
pub fn grown(rect: Rect, margin: u32) -> Rect {
    let shift = i32::try_from(margin).unwrap_or(0);
    Rect {
        x: rect.x.saturating_sub(shift),
        y: rect.y.saturating_sub(shift),
        width: rect.width.saturating_add(2 * margin),
        height: rect.height.saturating_add(2 * margin),
    }
}

/// Ramène la fenêtre normale dans l'écran où elle était ancrée si elle ne tient
/// entièrement dans aucun écran (lâchée à moitié hors de l'écran, par exemple).
fn keep_visible<R: Runtime>(
    window: &WebviewWindow<R>,
    restore: Restore,
    screen: Option<(Rect, f64)>,
) -> Restore {
    let rect = Rect {
        x: restore.position.x,
        y: restore.position.y,
        width: restore.size.width,
        height: restore.size.height,
    };
    let works: Vec<Rect> = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| {
            let area = m.work_area();
            Rect {
                x: area.position.x,
                y: area.position.y,
                width: area.size.width,
                height: area.size.height,
            }
        })
        .collect();
    if works.iter().any(|work| work.holds(rect)) {
        return restore;
    }
    let Some((work, _)) = screen else {
        return restore;
    };
    let moved = move_inside(work, rect);
    Restore {
        position: PhysicalPosition::new(moved.x, moved.y),
        ..restore
    }
}

/// Déplace `rect` le moins possible pour qu'il tienne dans `work` (collé en haut à
/// gauche s'il est plus grand).
pub fn move_inside(work: Rect, rect: Rect) -> Rect {
    let clamp = |start: i32, size: u32, area_start: i32, area_size: u32| {
        let end = i64::from(area_start) + i64::from(area_size) - i64::from(size);
        let end = i32::try_from(end).unwrap_or(i32::MAX).max(area_start);
        start.clamp(area_start, end)
    };
    Rect {
        x: clamp(rect.x, rect.width, work.x, work.width),
        y: clamp(rect.y, rect.height, work.y, work.height),
        ..rect
    }
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
    // Sous Linux (GTK), une fenêtre non redimensionnable garde sa taille : elle ne
    // pourrait plus devenir une encoche. Elle reste donc redimensionnable.
    let resizable = cfg!(target_os = "linux");
    let steps: [(&str, tauri::Result<()>); 6] = [
        ("plein écran", window.set_fullscreen(false)),
        ("taille minimale", window.set_min_size(None::<Size>)),
        ("redimensionnement", window.set_resizable(resizable)),
        ("premier plan", window.set_always_on_top(true)),
        ("barre des tâches", window.set_skip_taskbar(true)),
        ("ombre", window.set_shadow(false)),
    ];
    log_failures("ancrage", steps);
}

/// Sous Wayland avec le layer shell, colle la fenêtre au bord `edge` de l'écran
/// (voir `layer`). Ailleurs, rien à faire : la fenêtre se place elle-même.
/// Au démarrage, la fenêtre n'est pas encore affichée : c'est ici qu'elle l'est.
#[cfg(target_os = "linux")]
fn attach_layer<R: Runtime>(window: &WebviewWindow<R>, edge: DockEdge) {
    if !crate::display::native_wayland() {
        return;
    }
    let target = window.clone();
    // GTK ne s'utilise que depuis le fil principal.
    let result = window.run_on_main_thread(move || {
        let attached = target
            .gtk_window()
            .is_ok_and(|gtk_window| crate::layer::attach(&gtk_window, edge));
        if !attached {
            log::warn!("le bureau ne permet pas de coller l'encoche au bord de l'écran");
        }
        let _ = target.show();
    });
    if let Err(e) = result {
        log::warn!("impossible de coller l'encoche au bord de l'écran : {e}");
    }
}

#[cfg(not(target_os = "linux"))]
fn attach_layer<R: Runtime>(_window: &WebviewWindow<R>, _edge: DockEdge) {}

/// Rend à la fenêtre son aspect normal, sa taille et sa position d'avant l'ancrage.
/// Renvoie la position donnée à la fenêtre, si elle est connue.
fn leave_dock<R: Runtime>(
    window: &WebviewWindow<R>,
    restore: Option<Restore>,
    screen: Option<(Rect, f64)>,
) -> Option<PhysicalPosition<i32>> {
    pass_clicks(window, false);
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
            Some(restore.position)
        }
        (None, Some((work, scale))) => {
            let width = to_physical(NORMAL_WINDOW.0, scale);
            let height = to_physical(NORMAL_WINDOW.1, scale);
            let rect = fit(work, DockEdge::None, width, height);
            set_rect(window, rect);
            Some(PhysicalPosition::new(rect.x, rect.y))
        }
        (None, None) => {
            let _ = window.set_size(LogicalSize::new(NORMAL_WINDOW.0, NORMAL_WINDOW.1));
            let _ = window.center();
            None
        }
    }
}

/// Laisse passer les clics à travers la fenêtre (encoche repliée), ou non.
fn pass_clicks<R: Runtime>(window: &WebviewWindow<R>, through: bool) {
    if let Err(e) = window.set_ignore_cursor_events(through) {
        log::warn!("impossible de régler le passage des clics à travers l'encoche : {e}");
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
/// Avec `CLICK_THROUGH`, le panneau ouvert reçoit les clics et l'encoche repliée
/// les laisse passer. Sinon, la fenêtre grandit tout de suite à l'ouverture, et ne
/// rapetisse qu'à la fin de l'animation de fermeture (voir [`shrink`]).
fn set_expanded<R: Runtime>(app: &AppHandle<R>, generation: u64, expanded: bool) {
    let state = app.state::<DockState>();
    let (target, status) = {
        let mut inner = state.lock();
        if inner.generation != generation || inner.edge == DockEdge::None {
            return;
        }
        inner.expanded = expanded;
        (inner.target(CLICK_THROUGH), inner.status())
    };
    if let Some(window) = app.get_webview_window(MAIN) {
        if CLICK_THROUGH {
            pass_clicks(&window, !expanded);
        } else if let (true, Some(rect)) = (expanded, target) {
            set_rect(&window, rect);
        }
    }
    emit(app, status);
}

/// Remet la fenêtre à la taille de l'encoche si le panneau est toujours replié.
fn shrink<R: Runtime>(app: &AppHandle<R>, generation: u64) {
    let target = {
        let inner = app.state::<DockState>().inner().lock();
        if inner.generation != generation || inner.expanded {
            return;
        }
        inner.target(CLICK_THROUGH)
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
            let mut placement = Placement::default();
            loop {
                thread::sleep(Duration::from_millis(POLL_MS));
                clock += POLL_MS;
                let Some(window) = app.get_webview_window(MAIN) else {
                    return;
                };
                let actual = window_rect(&window);
                let (zone, pointer_inside, target) = {
                    let inner = app.state::<DockState>().inner().lock();
                    if inner.generation != generation {
                        return;
                    }
                    hover.expanded = inner.expanded;
                    let zone = actual.and_then(|r| inner.hover_rect(r));
                    (zone, inner.pointer_inside, inner.target(CLICK_THROUGH))
                };
                // Sous Wayland, la position n'est pas connue : le layer shell la tient.
                let placed_by_app = !crate::display::native_wayland();
                if let (true, Some(actual), Some(target)) = (placed_by_app, actual, target) {
                    match placement.check(actual, target, POLL_MS) {
                        Placed::Fine => {}
                        Placed::Retry => set_rect(&window, target),
                        Placed::Refused => {
                            // Une encoche hors du bord ne s'ouvrirait pas au survol
                            // et cacherait l'application : on revient à la normale.
                            app.state::<DockState>().lock().refused = true;
                            if let Err(e) = undock(&app) {
                                log::error!("impossible de détacher l'encoche : {e}");
                            }
                            return;
                        }
                    }
                }
                if !window.is_visible().unwrap_or(false) {
                    continue;
                }
                let inside = if CLICK_THROUGH {
                    let Ok(cursor) = app.cursor_position() else {
                        continue;
                    };
                    zone.is_some_and(|r| r.contains(cursor.x, cursor.y))
                } else {
                    pointer_inside
                };
                match hover.update(inside, POLL_MS) {
                    Some(true) => {
                        shrink_at = None;
                        set_expanded(&app, generation, true);
                    }
                    Some(false) => {
                        if !CLICK_THROUGH {
                            shrink_at = Some(clock + CLOSE_ANIMATION_MS);
                        }
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
    // Sous Wayland, même par XWayland, la souris et la fenêtre sont souvent lues
    // en 0,0, contre le bord : glisser la fenêtre ne peut pas l'ancrer.
    if crate::display::wayland_session() {
        return;
    }
    let now = Instant::now();
    let state = app.state::<DockState>();
    {
        let mut inner = state.lock();
        // Au démarrage, le bureau place lui-même la fenêtre : ce n'est pas un glisser.
        let starting = inner.started.is_some_and(|at| at.elapsed() < STARTUP_GRACE);
        let ignored = starting || inner.ignore_moves_until.is_some_and(|until| now < until);
        if inner.edge != DockEdge::None || !inner.dock_on_drag || ignored {
            return;
        }
        if let Some(drag) = inner.drag.as_mut() {
            drag.last_move = now;
            return;
        }
    }
    // Premier mouvement d'un glisser : on retient la fenêtre telle qu'elle était.
    let mut start = current_geometry(&window);
    if let (Some(geometry), Some(position)) = (start.as_mut(), state.lock().settled) {
        geometry.position = position;
    }
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
    // Un vrai glisser tient la fenêtre par sa barre de titre : la souris est dessus.
    // Sinon (souris inconnue, lue en 0,0 sous XWayland), ce n'est pas un glisser.
    let on_window = app
        .get_webview_window(MAIN)
        .and_then(|window| window_rect(&window))
        .is_some_and(|rect| grown(rect, DRAG_WINDOW_MARGIN).contains(cursor.x, cursor.y));
    if !on_window {
        return;
    }
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
        let position = app
            .get_webview_window(MAIN)
            .and_then(|window| window.outer_position().ok());
        app.state::<DockState>().lock().settled = position;
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

/// Indique si la fenêtre s'affiche d'elle-même : collée par le layer shell, elle
/// l'est à l'ancrage (voir [`attach_layer`]). Sinon, il faut l'afficher au démarrage.
pub fn shows_itself<R: Runtime>(app: &AppHandle<R>) -> bool {
    crate::display::native_wayland() && app.state::<DockState>().lock().edge != DockEdge::None
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
        inner.target(CLICK_THROUGH)
    };
    if let (Some(window), Some(rect)) = (app.get_webview_window(MAIN), target) {
        set_rect(&window, rect);
    }
    Ok(())
}

/// Longueur de l'encoche repliée, en pixels logiques : l'interface l'allonge quand
/// elle montre une activité en direct, et la remet à `NOTCH_LENGTH` ensuite. Seule
/// la zone qui ouvre le panneau change : la fenêtre garde sa taille.
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
    let target = {
        let mut inner = state.lock();
        inner.notch_length = Some(length.clamp(NOTCH_LENGTH_RANGE.0, NOTCH_LENGTH_RANGE.1));
        // Sans `CLICK_THROUGH`, la fenêtre repliée a la taille de l'encoche.
        if CLICK_THROUGH || inner.expanded {
            None
        } else {
            inner.target(false)
        }
    };
    if let (Some(window), Some(rect)) = (app.get_webview_window(MAIN), target) {
        set_rect(&window, rect);
    }
    Ok(())
}

/// La souris est entrée dans la fenêtre ancrée ou en est sortie, d'après
/// l'interface. Sert là où le Rust ne peut pas suivre la souris (Linux).
#[tauri::command]
pub fn dock_pointer(state: State<'_, DockState>, inside: bool) {
    state.lock().pointer_inside = inside;
}

/// Détache la fenêtre du bord de l'écran (menu de la zone de notification) : le
/// réglage est enregistré, puis appliqué.
pub fn undock<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    let store = app.state::<ConfigStore>();
    let mut config = store.get();
    if config.window.dock == DockEdge::None {
        return Ok(());
    }
    config.window.dock = DockEdge::None;
    let saved = store.set(config)?;
    apply(app, &saved.window);
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
            refused: false,
        };
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            r#"{"edge":"bottom","expanded":true,"refused":false}"#
        );
    }

    #[test]
    fn docked_window_keeps_the_panel_size_and_hovers_on_the_notch() {
        let mut inner = Inner {
            edge: DockEdge::Left,
            screen: Some((SCREEN, 1.0)),
            ..Inner::default()
        };
        let panel = fit(SCREEN, DockEdge::Left, 640, 360);
        let notch = fit(SCREEN, DockEdge::Left, 32, 180);
        // Fenêtre qui laisse passer les clics : toujours la taille du panneau.
        assert_eq!(inner.target(true), Some(panel));
        // Sinon (Linux) : la taille de l'encoche tant qu'elle est repliée.
        assert_eq!(inner.target(false), Some(notch));
        assert_eq!(inner.hover_rect(panel), Some(notch));
        inner.notch_length = Some(300.0);
        assert_eq!(inner.target(true), Some(panel));
        assert_eq!(
            inner.target(false),
            Some(fit(SCREEN, DockEdge::Left, 32, 300))
        );
        assert_eq!(
            inner.hover_rect(panel),
            Some(fit(SCREEN, DockEdge::Left, 32, 300))
        );
        inner.expanded = true;
        assert_eq!(inner.target(false), Some(panel));
        assert_eq!(inner.hover_rect(panel), Some(panel));
        inner.panel = Some((332.0, 700.0));
        let bigger = fit(SCREEN, DockEdge::Left, 332, 700);
        assert_eq!(inner.target(true), Some(bigger));
        assert_eq!((bigger.x, bigger.y), (0, 170));
        inner.edge = DockEdge::None;
        assert_eq!(inner.target(true), None);
        assert_eq!(inner.hover_rect(panel), None);
    }

    #[test]
    fn misplaced_window_is_placed_again_then_given_up() {
        let target = fit(SCREEN, DockEdge::Left, 32, 180);
        let centered = fit(SCREEN, DockEdge::None, 32, 180);
        let mut placement = Placement::default();
        assert_eq!(placement.check(target, target, POLL_MS), Placed::Fine);
        let mut tries = 0;
        let mut outcome = Placed::Fine;
        for _ in 0..200 {
            outcome = placement.check(centered, target, POLL_MS);
            match outcome {
                Placed::Retry => tries += 1,
                Placed::Refused => break,
                Placed::Fine => {}
            }
        }
        assert_eq!((tries, outcome), (PLACEMENT_TRIES, Placed::Refused));
        // Une fois abandonné, plus rien n'est demandé.
        assert_eq!(placement.check(centered, target, POLL_MS), Placed::Fine);
    }

    #[test]
    fn window_placed_after_a_while_is_fine() {
        let target = fit(SCREEN, DockEdge::Left, 32, 180);
        let centered = fit(SCREEN, DockEdge::None, 32, 180);
        let mut placement = Placement::default();
        assert_eq!(
            placement.check(centered, target, PLACEMENT_GRACE_MS),
            Placed::Retry
        );
        assert_eq!(placement.check(target, target, POLL_MS), Placed::Fine);
        assert_eq!(placement.check(centered, target, POLL_MS), Placed::Fine);
    }

    #[test]
    fn drag_margin_grows_the_window() {
        let rect = Rect {
            x: 10,
            y: 20,
            width: 100,
            height: 50,
        };
        let big = grown(rect, 24);
        assert_eq!((big.x, big.y, big.width, big.height), (-14, -4, 148, 98));
        assert!(big.contains(0.0, 0.0));
        assert!(!rect.contains(0.0, 0.0));
    }

    #[test]
    fn notch_follows_the_window_where_the_desktop_put_it() {
        // Le bureau a laissé la fenêtre au milieu de l'écran : l'encoche y est.
        let inner = Inner {
            edge: DockEdge::Left,
            screen: Some((SCREEN, 1.0)),
            ..Inner::default()
        };
        let centered = fit(SCREEN, DockEdge::None, 640, 360);
        let zone = inner.hover_rect(centered).unwrap();
        assert_eq!((zone.x, zone.width, zone.height), (centered.x, 32, 180));
        assert!(centered.holds(zone));
    }

    #[test]
    fn notch_is_drawn_inside_the_panel_window() {
        for edge in [
            DockEdge::Top,
            DockEdge::Bottom,
            DockEdge::Left,
            DockEdge::Right,
        ] {
            let (w, h) = notch_size(edge, NOTCH_LENGTH, 1.0);
            let notch = fit(SCREEN, edge, w, h);
            assert!(fit(SCREEN, edge, 640, 360).holds(notch), "{edge:?}");
        }
    }

    #[test]
    fn window_dropped_half_off_screen_comes_back_inside() {
        let off = Rect {
            x: -298,
            y: 283,
            width: 1024,
            height: 640,
        };
        assert!(!SCREEN.holds(off));
        let back = move_inside(SCREEN, off);
        assert_eq!((back.x, back.y, back.width), (0, 283, 1024));
        assert!(SCREEN.holds(back));
        let low = Rect { y: 900, ..off };
        assert_eq!(move_inside(SCREEN, low).y, 400);
        let huge = Rect {
            x: 50,
            y: 50,
            width: 4000,
            height: 3000,
        };
        assert_eq!(move_inside(SCREEN, huge), Rect { x: 0, y: 0, ..huge });
    }
}
