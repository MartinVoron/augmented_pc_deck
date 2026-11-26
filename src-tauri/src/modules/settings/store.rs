//! Lecture et écriture du fichier `config.json`.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::config::{self, Config, ParseError, CURRENT_VERSION};
use crate::error::{AppError, AppResult};

/// Nom du fichier de configuration, dans le dossier de l'application.
pub const FILE_NAME: &str = "config.json";

/// Configuration en mémoire, avec le chemin du fichier où elle est enregistrée.
pub struct ConfigStore {
    path: PathBuf,
    config: Mutex<Config>,
}

impl ConfigStore {
    /// Charge la configuration depuis `dir`, sans jamais bloquer le démarrage :
    /// - fichier absent : valeurs par défaut, enregistrées tout de suite ;
    /// - fichier abîmé ou trop récent : copie de sauvegarde, puis valeurs par défaut.
    pub fn load(dir: &Path) -> Self {
        let path = dir.join(FILE_NAME);
        let config = match fs::read_to_string(&path) {
            Ok(text) => match config::parse(&text) {
                Ok(config) => config,
                Err(error) => {
                    let reason = match &error {
                        ParseError::Invalid(detail) => format!("fichier abîmé ({detail})"),
                        ParseError::TooRecent(v) => format!("fichier en version {v}, trop récente"),
                    };
                    match backup(&path) {
                        Ok(copy) => log::warn!(
                            "configuration illisible : {reason}. Copie gardée dans {}",
                            copy.display()
                        ),
                        Err(e) => log::error!(
                            "configuration illisible : {reason}. Copie impossible : {e}"
                        ),
                    }
                    Config::default()
                }
            },
            Err(e) if e.kind() == ErrorKind::NotFound => {
                log::info!("première ouverture : création de {}", path.display());
                Config::default()
            }
            Err(e) => {
                log::error!("lecture de {} impossible : {e}", path.display());
                Config::default()
            }
        };

        let store = Self {
            path,
            config: Mutex::new(config),
        };
        if let Err(e) = store.write(&store.get()) {
            log::error!("{e}");
        }
        store
    }

    /// Chemin du fichier de configuration.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Copie de la configuration actuelle.
    pub fn get(&self) -> Config {
        self.lock().clone()
    }

    /// Remplace toute la configuration et l'enregistre.
    pub fn set(&self, mut config: Config) -> AppResult<Config> {
        config.version = CURRENT_VERSION;
        self.write(&config)?;
        *self.lock() = config.clone();
        Ok(config)
    }

    /// Modifie une partie de la configuration et l'enregistre.
    pub fn update(&self, change: impl FnOnce(&mut Config)) -> AppResult<Config> {
        let mut config = self.get();
        change(&mut config);
        self.set(config)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Config> {
        // Une panique pendant une modification ne doit pas bloquer la configuration.
        self.config.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Écrit le fichier sans risque de le laisser à moitié écrit :
    /// on écrit un fichier temporaire, puis on le renomme.
    fn write(&self, config: &Config) -> AppResult<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let text = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::Other(format!("configuration impossible à écrire : {e}")))?;
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, text)?;
        fs::rename(&temporary, &self.path)?;
        Ok(())
    }
}

/// Renomme un fichier abîmé en `config.json.<horodatage>.bak` pour ne rien perdre.
fn backup(path: &Path) -> std::io::Result<PathBuf> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let copy = path.with_extension(format!("json.{seconds}.bak"));
    fs::rename(path, &copy)?;
    Ok(copy)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backups(dir: &Path) -> Vec<PathBuf> {
        fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "bak"))
            .collect()
    }

    #[test]
    fn creates_default_file_on_first_launch() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::load(dir.path());

        assert_eq!(store.get(), Config::default());
        assert!(dir.path().join(FILE_NAME).exists());
    }

    #[test]
    fn saved_changes_are_found_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::load(dir.path());
        store
            .update(|config| config.general.notifications_enabled = false)
            .unwrap();

        let reloaded = ConfigStore::load(dir.path());
        assert!(!reloaded.get().general.notifications_enabled);
    }

    #[test]
    fn damaged_file_is_kept_aside_and_replaced_by_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(FILE_NAME), "{ abîmé").unwrap();

        let store = ConfigStore::load(dir.path());

        assert_eq!(store.get(), Config::default());
        let copies = backups(dir.path());
        assert_eq!(copies.len(), 1);
        assert_eq!(fs::read_to_string(&copies[0]).unwrap(), "{ abîmé");
        // Le nouveau fichier est lisible.
        assert!(config::parse(&fs::read_to_string(store.path()).unwrap()).is_ok());
    }

    #[test]
    fn file_from_newer_version_is_kept_aside() {
        let dir = tempfile::tempdir().unwrap();
        let newer = format!(r#"{{ "version": {} }}"#, CURRENT_VERSION + 1);
        fs::write(dir.path().join(FILE_NAME), &newer).unwrap();

        let store = ConfigStore::load(dir.path());

        assert_eq!(store.get(), Config::default());
        assert_eq!(backups(dir.path()).len(), 1);
    }

    #[test]
    fn no_temporary_file_left_after_saving() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::load(dir.path());
        store.set(Config::default()).unwrap();

        assert!(!dir.path().join("config.json.tmp").exists());
    }
}
