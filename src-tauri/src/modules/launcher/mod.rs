//! Lanceur : ouvrir une application, un fichier, un dossier ou une URL depuis une tuile.
//!
//! - Un programme (`.exe`, `.bat`, `.cmd`, `.com`) est lancé directement, avec ses arguments,
//!   depuis son propre dossier.
//! - Tout le reste (fichier, dossier, raccourci `.lnk`, URL) est ouvert par Windows avec
//!   l'application associée, comme un double-clic dans l'Explorateur.
//! - L'icône d'un programme ou d'un fichier est extraite par Windows (voir [`icon`]).

mod icon;

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, AppResult};

/// Nature d'une cible, déduite du texte saisi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Adresse avec un schéma : `https://…`, `mailto:…`, `spotify:…`
    Url(String),
    /// Programme à lancer avec des arguments.
    Program(PathBuf),
    /// Fichier, dossier ou raccourci, ouvert avec l'application associée.
    Path(PathBuf),
}

/// Extensions lancées comme des programmes.
const PROGRAM_EXTENSIONS: [&str; 4] = ["exe", "bat", "cmd", "com"];

impl Target {
    /// Analyse le texte d'une cible. Un schéma d'une seule lettre (`C:`) est un lecteur,
    /// pas une URL.
    pub fn parse(text: &str) -> AppResult<Self> {
        let text = text.trim().trim_matches('"');
        if text.is_empty() {
            return Err(AppError::InvalidInput(
                "la cible du raccourci est vide".into(),
            ));
        }
        if let Some((scheme, _)) = text.split_once(':') {
            let is_scheme = scheme.len() > 1
                && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
            if is_scheme {
                return Ok(Self::Url(text.to_string()));
            }
        }
        let path = PathBuf::from(text);
        let is_program = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| PROGRAM_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()));
        Ok(if is_program {
            Self::Program(path)
        } else {
            Self::Path(path)
        })
    }
}

/// Découpe les arguments de lancement comme une ligne de commande :
/// les espaces séparent les arguments, les guillemets regroupent un argument qui en contient.
pub fn split_args(text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut has_arg = false;
    for c in text.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                has_arg = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if has_arg {
                    args.push(std::mem::take(&mut current));
                    has_arg = false;
                }
            }
            c => {
                current.push(c);
                has_arg = true;
            }
        }
    }
    if has_arg {
        args.push(current);
    }
    args
}

/// Vérifie qu'un fichier ou un dossier existe toujours.
fn ensure_exists(path: &Path) -> AppResult<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(AppError::NotFound(format!("« {} »", path.display())))
    }
}

/// Traduit une erreur de lancement en message clair.
fn launch_error(target: &str, error: std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::NotFound => AppError::NotFound(format!("« {target} »")),
        std::io::ErrorKind::PermissionDenied => {
            AppError::PermissionDenied(format!("Windows refuse d'ouvrir « {target} »"))
        }
        _ => AppError::Other(format!("impossible d'ouvrir « {target} » : {error}")),
    }
}

/// Lance la cible d'un raccourci.
#[tauri::command]
pub fn launcher_launch(target: String, args: Option<String>) -> AppResult<()> {
    let parsed = Target::parse(&target)?;
    log::info!("lancement de {parsed:?}");
    match parsed {
        Target::Url(url) => open::that_detached(&url).map_err(|e| launch_error(&url, e)),
        Target::Path(path) => {
            ensure_exists(&path)?;
            open::that_detached(&path).map_err(|e| launch_error(&target, e))
        }
        Target::Program(path) => {
            ensure_exists(&path)?;
            let mut command = Command::new(&path);
            command.args(split_args(args.as_deref().unwrap_or_default()));
            if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
                command.current_dir(dir);
            }
            command
                .spawn()
                .map(|_| ())
                .map_err(|e| launch_error(&target, e))
        }
    }
}

/// Renvoie l'icône d'un programme, d'un fichier ou d'un dossier, en image PNG
/// (`data:image/png;base64,…`), ou `None` si la cible n'a pas d'icône (URL, fichier absent).
#[tauri::command]
pub async fn launcher_icon(target: String) -> AppResult<Option<String>> {
    let path = match Target::parse(&target)? {
        Target::Url(_) => return Ok(None),
        Target::Program(path) | Target::Path(path) => path,
    };
    if !path.exists() {
        return Ok(None);
    }
    tauri::async_runtime::spawn_blocking(move || icon::extract_data_url(&path))
        .await
        .map_err(|e| AppError::Other(format!("extraction de l'icône interrompue : {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_urls_and_drive_letters() {
        assert_eq!(
            Target::parse("https://github.com").unwrap(),
            Target::Url("https://github.com".into())
        );
        assert_eq!(
            Target::parse("mailto:moi@exemple.fr").unwrap(),
            Target::Url("mailto:moi@exemple.fr".into())
        );
        assert_eq!(
            Target::parse(r"C:\Windows\notepad.exe").unwrap(),
            Target::Program(PathBuf::from(r"C:\Windows\notepad.exe"))
        );
        assert_eq!(
            Target::parse(r#""D:\Musique\""#).unwrap(),
            Target::Path(PathBuf::from(r"D:\Musique\"))
        );
    }

    #[test]
    fn program_extensions_ignore_case() {
        assert!(matches!(
            Target::parse(r"C:\outils\Build.CMD").unwrap(),
            Target::Program(_)
        ));
        assert!(matches!(
            Target::parse(r"C:\Bureau\Jeu.lnk").unwrap(),
            Target::Path(_)
        ));
    }

    #[test]
    fn rejects_empty_target() {
        let error = Target::parse("   ").unwrap_err();
        assert_eq!(error.code(), "invalid_input");
    }

    #[test]
    fn splits_arguments_with_quotes() {
        assert_eq!(
            split_args(r#"--profil "Mon profil"  -v """#),
            vec!["--profil", "Mon profil", "-v", ""]
        );
        assert!(split_args("   ").is_empty());
    }

    #[test]
    fn missing_target_gives_a_clear_message() {
        let error = launcher_launch("/dossier/qui/n/existe/pas.exe".into(), None).unwrap_err();
        assert_eq!(error.code(), "not_found");
        assert!(error.to_string().contains("est introuvable"));
    }

    #[test]
    fn url_and_missing_file_have_no_icon() {
        let icon = |target: &str| tauri::async_runtime::block_on(launcher_icon(target.into()));
        assert_eq!(icon("https://exemple.fr").unwrap(), None);
        assert_eq!(icon("/pas/la.exe").unwrap(), None);
    }
}
