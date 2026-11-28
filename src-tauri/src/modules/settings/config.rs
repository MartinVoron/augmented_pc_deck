//! Contenu de la configuration et migrations entre les versions du format.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::weather::Place;

/// Version actuelle du format du fichier de configuration.
/// À augmenter à chaque changement de format, en ajoutant une étape dans [`migrate`].
pub const CURRENT_VERSION: u32 = 3;

/// Configuration complète de l'application, enregistrée dans `config.json`.
///
/// `#[serde(default)]` : un champ absent du fichier prend sa valeur par défaut,
/// ce qui permet d'ajouter un réglage sans casser les fichiers existants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub version: u32,
    pub general: GeneralSettings,
    pub window: WindowSettings,
    pub layout: Layout,
    pub github: GithubSettings,
    pub gmail: GmailSettings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            general: GeneralSettings::default(),
            window: WindowSettings::default(),
            layout: Layout::default(),
            github: GithubSettings::default(),
            gmail: GmailSettings::default(),
        }
    }
}

/// Réglages généraux.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GeneralSettings {
    /// Langue de l'interface (`fr` uniquement pour l'instant).
    pub language: String,
    /// Notifications Windows activées pour toute l'application.
    pub notifications_enabled: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "fr".into(),
            notifications_enabled: true,
        }
    }
}

/// Options d'affichage de la fenêtre. Sa taille et sa position sont retenues à part,
/// par le plugin `window-state` (fichier `.window-state.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowSettings {
    /// Fenêtre toujours au-dessus des autres.
    pub always_on_top: bool,
    /// Fenêtre en plein écran.
    pub fullscreen: bool,
    /// Bord de l'écran où la fenêtre est ancrée en encoche (`none` : fenêtre normale).
    pub dock: DockEdge,
    /// Apparence de l'encoche quand la fenêtre ancrée est repliée.
    pub notch_style: NotchStyle,
    /// Lâcher la fenêtre avec la souris contre un bord de l'écran l'y ancre.
    pub dock_on_drag: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            always_on_top: false,
            fullscreen: false,
            dock: DockEdge::None,
            notch_style: NotchStyle::Logo,
            dock_on_drag: true,
        }
    }
}

/// Bord de l'écran où la fenêtre est ancrée. Une valeur inconnue revient à `none`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DockEdge {
    Top,
    Bottom,
    Left,
    Right,
    /// Fenêtre normale, non ancrée.
    #[default]
    #[serde(other)]
    None,
}

/// Apparence de l'encoche repliée. Une valeur inconnue revient à `logo`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotchStyle {
    /// Encoche entièrement noire.
    Black,
    /// Encoche noire avec le logo de l'application.
    #[default]
    #[serde(other)]
    Logo,
}

/// Réglages du module GitHub.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GithubSettings {
    /// Intervalle de rafraîchissement de l'activité, en minutes.
    pub refresh_minutes: u32,
}

impl Default for GithubSettings {
    fn default() -> Self {
        Self { refresh_minutes: 5 }
    }
}

/// Réglages du module Gmail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GmailSettings {
    /// Intervalle de rafraîchissement des e-mails non lus, en minutes.
    pub refresh_minutes: u32,
    /// Notification Windows pour chaque nouveau message.
    pub notify_new: bool,
}

impl Default for GmailSettings {
    fn default() -> Self {
        Self {
            refresh_minutes: 2,
            notify_new: true,
        }
    }
}

/// Disposition des tuiles, rangées en pages (onglets).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Layout {
    pub pages: Vec<Page>,
}

/// Une page de tuiles. Les identifiants des tuiles sont uniques dans toutes les pages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    /// Nom affiché dans l'onglet.
    pub name: String,
    /// Écran d'accueil : la page affichée à l'ouverture, qui ne peut pas être supprimée.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub home: bool,
    #[serde(default)]
    pub tiles: Vec<TileConfig>,
}

/// Une tuile de la grille.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileConfig {
    pub id: String,
    /// Module affiché : `launcher`, `processes`, `timers`, `todos`, `music`…
    pub kind: String,
    pub title: String,
    /// Taille : `1x1`, `2x1` ou `2x2`.
    pub size: String,
    /// Raccourcis d'une tuile du lanceur (vide pour les autres modules).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shortcuts: Vec<Shortcut>,
    /// Comptes à rebours d'une tuile des minuteurs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timers: Vec<TimerSettings>,
    /// Durées du mode Pomodoro d'une tuile des minuteurs (valeurs par défaut si absent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pomodoro: Option<PomodoroSettings>,
    /// Tâches d'une tuile de la liste de tâches, dans l'ordre affiché.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub todos: Vec<Todo>,
    /// Masque les tâches terminées d'une tuile de la liste de tâches.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hide_done: bool,
    /// Affiche les secondes d'une tuile Horloge.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub show_seconds: bool,
    /// Ville d'une tuile Météo (Caen si absente).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Place>,
}

/// Une tâche de la liste.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub done: bool,
}

/// Durées du mode Pomodoro, en minutes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroSettings {
    pub work_mins: u32,
    pub short_break_mins: u32,
    pub long_break_mins: u32,
    /// Nombre de séances de travail avant une longue pause.
    pub sessions_before_long_break: u32,
}

/// Un compte à rebours enregistré (son nom et sa durée ; l'état en cours n'est pas gardé).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSettings {
    pub id: String,
    pub name: String,
    /// Durée, en secondes.
    pub duration_secs: u64,
}

/// Un raccourci de la tuile du lanceur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shortcut {
    pub id: String,
    /// Nom affiché sous l'icône.
    pub name: String,
    /// Programme, fichier, dossier ou URL à ouvrir.
    pub target: String,
    /// Arguments de lancement d'un programme, comme sur une ligne de commande.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub args: String,
    /// Image choisie à la place de l'icône extraite de la cible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl TileConfig {
    fn new(kind: &str, title: &str, size: &str) -> Self {
        Self {
            id: format!("{kind}-1"),
            kind: kind.into(),
            title: title.into(),
            size: size.into(),
            shortcuts: Vec::new(),
            timers: Vec::new(),
            pomodoro: None,
            todos: Vec::new(),
            hide_done: false,
            show_seconds: false,
            location: None,
        }
    }
}

impl Default for Layout {
    /// Disposition proposée au premier lancement : l'écran d'accueil (heure, météo
    /// à Caen, tâches à faire), puis une page avec les autres modules.
    fn default() -> Self {
        Self {
            pages: vec![
                Page {
                    id: HOME_PAGE_ID.into(),
                    name: HOME_PAGE_NAME.into(),
                    home: true,
                    tiles: vec![
                        TileConfig::new("clock", "Horloge", "2x1"),
                        TileConfig::new("weather", "Météo", "2x1"),
                        TileConfig::new("todos", "Tâches à faire", "2x2"),
                    ],
                },
                Page {
                    id: "page-1".into(),
                    name: "Principal".into(),
                    home: false,
                    tiles: vec![
                        TileConfig::new("launcher", "Raccourcis", "2x1"),
                        TileConfig::new("processes", "Processeur et mémoire", "2x1"),
                        TileConfig::new("music", "Musique", "2x2"),
                        TileConfig::new("timers", "Minuteurs", "1x1"),
                        TileConfig::new("github", "GitHub", "1x1"),
                        TileConfig::new("gmail", "Gmail", "1x1"),
                        TileConfig::new("claude", "Claude", "1x1"),
                    ],
                },
            ],
        }
    }
}

/// Identifiant et nom de l'écran d'accueil.
const HOME_PAGE_ID: &str = "accueil";
const HOME_PAGE_NAME: &str = "Accueil";

/// Raison pour laquelle un fichier n'a pas pu être lu.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Le fichier n'est pas du JSON valide, ou son contenu est incohérent.
    Invalid(String),
    /// Le fichier vient d'une version plus récente de l'application.
    TooRecent(u32),
}

/// Lit le texte d'un fichier de configuration, en le migrant si besoin.
pub fn parse(text: &str) -> Result<Config, ParseError> {
    let mut value: Value =
        serde_json::from_str(text).map_err(|e| ParseError::Invalid(e.to_string()))?;
    if !value.is_object() {
        return Err(ParseError::Invalid(
            "le fichier ne contient pas un objet JSON".into(),
        ));
    }
    migrate(&mut value)?;
    serde_json::from_value(value).map_err(|e| ParseError::Invalid(e.to_string()))
}

/// Fait passer un fichier de son ancienne version à [`CURRENT_VERSION`], étape par étape.
///
/// Pour un nouveau format : ajouter une étape `n => …` qui passe de la version `n`
/// à `n + 1`, puis augmenter [`CURRENT_VERSION`]. Chaque étape ne connaît que la
/// version précédente.
fn migrate(value: &mut Value) -> Result<(), ParseError> {
    // Un fichier sans numéro de version est traité comme la version 1.
    let mut version = value
        .get("version")
        .and_then(Value::as_u64)
        .map_or(1, |v| u32::try_from(v).unwrap_or(u32::MAX));

    if version > CURRENT_VERSION {
        return Err(ParseError::TooRecent(version));
    }
    while version < CURRENT_VERSION {
        match version {
            1 => tiles_into_pages(value),
            2 => add_home_page(value),
            _ => {}
        }
        version += 1;
    }
    value["version"] = Value::from(CURRENT_VERSION);
    Ok(())
}

/// Version 1 → 2 : la liste unique de tuiles devient la première page.
fn tiles_into_pages(value: &mut Value) {
    let Some(layout) = value.get_mut("layout").and_then(Value::as_object_mut) else {
        return;
    };
    if let Some(tiles) = layout.remove("tiles") {
        layout.insert(
            "pages".into(),
            serde_json::json!([{ "id": "page-1", "name": "Principal", "tiles": tiles }]),
        );
    }
}

/// Version 2 → 3 : ajoute l'écran d'accueil en tête (heure, météo, tâches).
/// La première liste de tâches existante y est déplacée, avec ses tâches.
fn add_home_page(value: &mut Value) {
    let Some(pages) = value
        .get_mut("layout")
        .and_then(|l| l.get_mut("pages"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    if pages
        .iter()
        .any(|p| p.get("home") == Some(&Value::Bool(true)))
    {
        return;
    }
    let ids: Vec<String> = pages
        .iter()
        .filter_map(|p| p.get("tiles").and_then(Value::as_array))
        .flatten()
        .filter_map(|t| t.get("id").and_then(Value::as_str).map(String::from))
        .collect();
    let free_id = |kind: &str| {
        (1..)
            .map(|n| format!("{kind}-{n}"))
            .find(|id| !ids.contains(id))
            .unwrap_or_default()
    };
    let tile = |kind: &str, title: &str, size: &str| serde_json::json!({ "id": free_id(kind), "kind": kind, "title": title, "size": size });
    let clock = tile("clock", "Horloge", "2x1");
    let weather = tile("weather", "Météo", "2x1");

    // Déplace la première liste de tâches, pour garder les tâches déjà notées.
    let mut todos = None;
    for page in pages.iter_mut() {
        let Some(tiles) = page.get_mut("tiles").and_then(Value::as_array_mut) else {
            continue;
        };
        if let Some(index) = tiles
            .iter()
            .position(|t| t.get("kind").and_then(Value::as_str) == Some("todos"))
        {
            let mut moved = tiles.remove(index);
            moved["size"] = Value::from("2x2");
            todos = Some(moved);
            break;
        }
    }
    let todos = todos.unwrap_or_else(|| tile("todos", "Tâches à faire", "2x2"));

    let id_taken = pages
        .iter()
        .any(|p| p.get("id").and_then(Value::as_str) == Some(HOME_PAGE_ID));
    let page_id = if id_taken {
        format!("{HOME_PAGE_ID}-{}", pages.len() + 1)
    } else {
        HOME_PAGE_ID.to_string()
    };
    pages.insert(
        0,
        serde_json::json!({
            "id": page_id,
            "name": HOME_PAGE_NAME,
            "home": true,
            "tiles": [clock, weather, todos],
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_opens_on_the_home_page_and_has_every_module() {
        let config = Config::default();
        assert_eq!(config.version, CURRENT_VERSION);
        assert!(config.general.notifications_enabled);
        assert_eq!(config.general.language, "fr");
        assert!(!config.window.always_on_top && !config.window.fullscreen);
        let pages = &config.layout.pages;
        assert!(pages[0].home);
        let kinds: Vec<&str> = pages[0].tiles.iter().map(|t| t.kind.as_str()).collect();
        assert_eq!(kinds, ["clock", "weather", "todos"]);
        assert_eq!(pages.iter().map(|p| p.tiles.len()).sum::<usize>(), 10);
    }

    #[test]
    fn written_config_reads_back_identical() {
        let mut config = Config::default();
        config.general.notifications_enabled = false;
        let text = serde_json::to_string_pretty(&config).unwrap();
        assert!(text.contains("\"notificationsEnabled\": false"));
        assert_eq!(parse(&text).unwrap(), config);
    }

    #[test]
    fn window_dock_reads_and_falls_back_to_a_normal_window() {
        let config =
            parse(r#"{ "version": 3, "window": { "dock": "left", "notchStyle": "black" } }"#)
                .unwrap();
        assert_eq!(config.window.dock, DockEdge::Left);
        assert_eq!(config.window.notch_style, NotchStyle::Black);
        let written = serde_json::to_string(&config).unwrap();
        assert!(written.contains(r#""dock":"left""#));
        assert!(written.contains(r#""notchStyle":"black""#));

        // Ancienne configuration ou valeur inconnue : fenêtre normale, encoche avec logo.
        let old = parse(r#"{ "version": 3, "window": { "alwaysOnTop": true } }"#).unwrap();
        assert_eq!(old.window.dock, DockEdge::None);
        assert_eq!(old.window.notch_style, NotchStyle::Logo);
        assert!(old.window.dock_on_drag);
        let odd =
            parse(r#"{ "version": 3, "window": { "dock": "milieu", "notchStyle": "rose" } }"#)
                .unwrap();
        assert_eq!(odd.window.dock, DockEdge::None);
        assert_eq!(odd.window.notch_style, NotchStyle::Logo);
    }

    #[test]
    fn missing_fields_take_default_values() {
        let config = parse(r#"{ "version": 1, "general": {} }"#).unwrap();
        assert!(config.general.notifications_enabled);
        assert_eq!(config.layout, Layout::default());
    }

    #[test]
    fn launcher_tile_keeps_its_shortcuts() {
        let text = r#"{ "version": 3, "layout": { "pages": [{ "id": "p", "name": "P", "tiles": [{
            "id": "launcher-1", "kind": "launcher", "title": "Raccourcis", "size": "2x1",
            "shortcuts": [{ "id": "s1", "name": "Bloc-notes", "target": "C:\\Windows\\notepad.exe" }]
        }, { "id": "timers-1", "kind": "timers", "title": "Minuteurs", "size": "1x1" }] }] } }"#;
        let config = parse(text).unwrap();
        let shortcut = &config.layout.pages[0].tiles[0].shortcuts[0];
        assert_eq!(shortcut.name, "Bloc-notes");
        assert_eq!(shortcut.args, "");
        assert!(config.layout.pages[0].tiles[1].shortcuts.is_empty());
        // Une tuile sans raccourci n'écrit pas de liste vide dans le fichier.
        let written = serde_json::to_string(&config).unwrap();
        assert_eq!(written.matches("shortcuts").count(), 1);
    }

    #[test]
    fn timers_tile_keeps_its_countdowns() {
        let text = r#"{ "version": 3, "layout": { "pages": [{ "id": "p", "name": "P", "tiles": [{
            "id": "timers-1", "kind": "timers", "title": "Minuteurs", "size": "1x1",
            "timers": [{ "id": "minuteur-1", "name": "Thé", "durationSecs": 240 }]
        }] }] } }"#;
        let config = parse(text).unwrap();
        assert_eq!(config.layout.pages[0].tiles[0].timers[0].duration_secs, 240);
        assert_eq!(config.layout.pages[0].tiles[0].pomodoro, None);
    }

    #[test]
    fn todos_tile_keeps_its_tasks_and_option() {
        let text = r#"{ "version": 3, "layout": { "pages": [{ "id": "p", "name": "P", "tiles": [{
            "id": "todos-1", "kind": "todos", "title": "Tâches", "size": "2x2", "hideDone": true,
            "todos": [{ "id": "t1", "text": "Appeler le garage", "done": true }, { "id": "t2", "text": "Courses" }]
        }] }] } }"#;
        let config = parse(text).unwrap();
        let tile = &config.layout.pages[0].tiles[0];
        assert!(tile.hide_done);
        assert!(tile.todos[0].done && !tile.todos[1].done);
        let written = serde_json::to_string(&config).unwrap();
        assert_eq!(parse(&written).unwrap(), config);
    }

    #[test]
    fn clock_tile_keeps_its_seconds_option() {
        let text = r#"{ "version": 3, "layout": { "pages": [{ "id": "p", "name": "P", "tiles": [{
            "id": "clock-1", "kind": "clock", "title": "Horloge", "size": "2x1", "showSeconds": true
        }] }] } }"#;
        let config = parse(text).unwrap();
        assert!(config.layout.pages[0].tiles[0].show_seconds);
        let written = serde_json::to_string(&config).unwrap();
        assert_eq!(parse(&written).unwrap(), config);
    }

    #[test]
    fn weather_tile_keeps_its_city() {
        let text = r#"{ "version": 3, "layout": { "pages": [{ "id": "p", "name": "P", "tiles": [{
            "id": "weather-1", "kind": "weather", "title": "Météo", "size": "2x1",
            "location": { "name": "Caen", "latitude": 49.18, "longitude": -0.36, "detail": "Normandie, France" }
        }] }] } }"#;
        let config = parse(text).unwrap();
        assert_eq!(
            config.layout.pages[0].tiles[0]
                .location
                .as_ref()
                .unwrap()
                .name,
            "Caen"
        );
        let written = serde_json::to_string(&config).unwrap();
        assert_eq!(parse(&written).unwrap(), config);
    }

    #[test]
    fn version_1_tiles_become_a_page_after_the_home_page() {
        let text = r#"{ "version": 1, "layout": { "tiles": [
            { "id": "todos-1", "kind": "todos", "title": "Tâches", "size": "2x2",
              "todos": [{ "id": "t1", "text": "Courses" }] }
        ] } }"#;
        let config = parse(text).unwrap();
        assert_eq!(config.version, CURRENT_VERSION);
        let names: Vec<&str> = config
            .layout
            .pages
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(names, ["Accueil", "Principal"]);
        // La liste de tâches passe sur l'écran d'accueil, avec ses tâches.
        let home = &config.layout.pages[0];
        assert!(home.home);
        assert_eq!(home.tiles[2].todos[0].text, "Courses");
        assert!(config.layout.pages[1].tiles.is_empty());
    }

    #[test]
    fn version_2_gets_a_home_page_with_unique_tile_ids() {
        let text = r#"{ "version": 2, "layout": { "pages": [
            { "id": "page-1", "name": "Travail", "tiles": [
                { "id": "clock-1", "kind": "clock", "title": "Horloge", "size": "1x1" },
                { "id": "todos-1", "kind": "todos", "title": "Courses", "size": "1x1",
                  "todos": [{ "id": "t1", "text": "Pain" }] }] }
        ] } }"#;
        let config = parse(text).unwrap();
        let home = &config.layout.pages[0];
        assert_eq!(
            (home.id.as_str(), home.name.as_str()),
            ("accueil", "Accueil")
        );
        let ids: Vec<&str> = home.tiles.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, ["clock-2", "weather-1", "todos-1"]);
        assert_eq!(home.tiles[2].title, "Courses");
        assert_eq!(home.tiles[2].size, "2x2");
        assert_eq!(config.layout.pages[1].tiles.len(), 1);
    }

    #[test]
    fn version_2_without_todos_gets_a_new_list() {
        let text = r#"{ "version": 2, "layout": { "pages": [{ "id": "p", "name": "P" }] } }"#;
        let config = parse(text).unwrap();
        assert_eq!(config.layout.pages[0].tiles[2].id, "todos-1");
        assert_eq!(config.layout.pages[0].tiles[2].title, "Tâches à faire");
    }

    #[test]
    fn reads_several_pages() {
        let text = r#"{ "version": 3, "layout": { "pages": [
            { "id": "page-1", "name": "Travail", "tiles": [
                { "id": "github-1", "kind": "github", "title": "GitHub", "size": "1x1" }] },
            { "id": "page-2", "name": "Vide" }
        ] } }"#;
        let config = parse(text).unwrap();
        assert_eq!(config.layout.pages[0].tiles[0].kind, "github");
        assert!(config.layout.pages[1].tiles.is_empty());
        let written = serde_json::to_string(&config).unwrap();
        assert_eq!(parse(&written).unwrap(), config);
    }

    #[test]
    fn file_without_version_is_read_as_version_1() {
        let config = parse(r#"{ "general": { "notificationsEnabled": false } }"#).unwrap();
        assert_eq!(config.version, CURRENT_VERSION);
        assert!(!config.general.notifications_enabled);
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(matches!(
            parse("{ pas du json"),
            Err(ParseError::Invalid(_))
        ));
        assert!(matches!(parse("[1, 2]"), Err(ParseError::Invalid(_))));
    }

    #[test]
    fn rejects_file_from_newer_version() {
        let text = format!(r#"{{ "version": {} }}"#, CURRENT_VERSION + 1);
        assert_eq!(
            parse(&text),
            Err(ParseError::TooRecent(CURRENT_VERSION + 1))
        );
    }
}
