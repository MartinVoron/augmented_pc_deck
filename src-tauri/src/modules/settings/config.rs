//! Contenu de la configuration et migrations entre les versions du format.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Version actuelle du format du fichier de configuration.
/// À augmenter à chaque changement de format, en ajoutant une étape dans [`migrate`].
pub const CURRENT_VERSION: u32 = 1;

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            general: GeneralSettings::default(),
            window: WindowSettings::default(),
            layout: Layout::default(),
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowSettings {
    /// Fenêtre toujours au-dessus des autres.
    pub always_on_top: bool,
    /// Fenêtre en plein écran.
    pub fullscreen: bool,
}

/// Disposition de la grille de tuiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Layout {
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
        }
    }
}

impl Default for Layout {
    /// Disposition proposée au premier lancement : une tuile par module.
    fn default() -> Self {
        Self {
            tiles: vec![
                TileConfig::new("launcher", "Raccourcis", "2x1"),
                TileConfig::new("processes", "Processeur et mémoire", "2x1"),
                TileConfig::new("music", "Musique", "2x2"),
                TileConfig::new("timers", "Minuteurs", "1x1"),
                TileConfig::new("todos", "Tâches", "2x2"),
                TileConfig::new("github", "GitHub", "1x1"),
                TileConfig::new("gmail", "Gmail", "1x1"),
                TileConfig::new("claude", "Claude", "1x1"),
            ],
        }
    }
}

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
/// Pour un futur format 2 : ajouter `1 => { /* transformer value */ }` puis passer
/// [`CURRENT_VERSION`] à 2. Chaque étape ne connaît que la version précédente.
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
        // Aucune migration pour l'instant : la version 1 est la première.
        version += 1;
    }
    value["version"] = Value::from(CURRENT_VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_current_version_and_one_tile_per_module() {
        let config = Config::default();
        assert_eq!(config.version, CURRENT_VERSION);
        assert!(config.general.notifications_enabled);
        assert_eq!(config.general.language, "fr");
        assert!(!config.window.always_on_top && !config.window.fullscreen);
        assert_eq!(config.layout.tiles.len(), 8);
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
    fn missing_fields_take_default_values() {
        let config = parse(r#"{ "version": 1, "general": {} }"#).unwrap();
        assert!(config.general.notifications_enabled);
        assert_eq!(config.layout, Layout::default());
    }

    #[test]
    fn launcher_tile_keeps_its_shortcuts() {
        let text = r#"{ "layout": { "tiles": [{
            "id": "launcher-1", "kind": "launcher", "title": "Raccourcis", "size": "2x1",
            "shortcuts": [{ "id": "s1", "name": "Bloc-notes", "target": "C:\\Windows\\notepad.exe" }]
        }, { "id": "timers-1", "kind": "timers", "title": "Minuteurs", "size": "1x1" }] } }"#;
        let config = parse(text).unwrap();
        let shortcut = &config.layout.tiles[0].shortcuts[0];
        assert_eq!(shortcut.name, "Bloc-notes");
        assert_eq!(shortcut.args, "");
        assert!(config.layout.tiles[1].shortcuts.is_empty());
        // Une tuile sans raccourci n'écrit pas de liste vide dans le fichier.
        let written = serde_json::to_string(&config).unwrap();
        assert_eq!(written.matches("shortcuts").count(), 1);
    }

    #[test]
    fn timers_tile_keeps_its_countdowns() {
        let text = r#"{ "layout": { "tiles": [{
            "id": "timers-1", "kind": "timers", "title": "Minuteurs", "size": "1x1",
            "timers": [{ "id": "minuteur-1", "name": "Thé", "durationSecs": 240 }]
        }] } }"#;
        let config = parse(text).unwrap();
        assert_eq!(config.layout.tiles[0].timers[0].duration_secs, 240);
        assert_eq!(config.layout.tiles[0].pomodoro, None);
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
