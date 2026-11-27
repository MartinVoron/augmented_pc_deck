//! Lanceur : ouvrir une application, un fichier, un dossier ou une URL depuis une tuile.
//!
//! - Un programme (`.exe`, `.bat`, `.cmd`, `.com`) est lancé directement, avec ses arguments,
//!   depuis son propre dossier.
//! - Tout le reste (fichier, dossier, raccourci `.lnk`, URL) est ouvert par Windows avec
//!   l'application associée, comme un double-clic dans l'Explorateur.
//! - L'icône d'un programme ou d'un fichier est extraite par Windows (voir [`icon`]).
//!
//! Sous Linux :
//! - un fichier exécutable (sans extension, `.AppImage`, script…) est lancé comme un programme ;
//! - un fichier `.desktop` (application du menu) lance la commande qu'il décrit, et donne
//!   son icône (voir [`desktop`]) ;
//! - le reste est ouvert avec l'application associée (`xdg-open`).

mod desktop;
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
    /// Application décrite par un fichier `.desktop` (Linux).
    Desktop(PathBuf),
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
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);
        Ok(match extension.as_deref() {
            Some(e) if PROGRAM_EXTENSIONS.contains(&e) => Self::Program(path),
            Some("desktop") if cfg!(target_os = "linux") => Self::Desktop(path),
            _ => Self::Path(path),
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

/// Fichier qu'on peut exécuter (droit `x`), hors dossiers. Toujours faux sous Windows,
/// où les programmes se reconnaissent à leur extension.
fn is_executable_file(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

/// Lance un programme avec ses arguments, depuis `dir` s'il est donné.
fn spawn(target: &str, program: &Path, args: Vec<String>, dir: Option<&Path>) -> AppResult<()> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(dir) = dir.filter(|d| !d.as_os_str().is_empty()) {
        command.current_dir(dir);
    }
    command
        .spawn()
        .map(|_| ())
        .map_err(|e| launch_error(target, e))
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
        Target::Path(path) if !is_executable_file(&path) => {
            ensure_exists(&path)?;
            open::that_detached(&path).map_err(|e| launch_error(&target, e))
        }
        Target::Program(path) | Target::Path(path) => {
            ensure_exists(&path)?;
            let args = split_args(args.as_deref().unwrap_or_default());
            spawn(&target, &path, args, path.parent())
        }
        Target::Desktop(path) => {
            ensure_exists(&path)?;
            let entry = desktop::parse(&std::fs::read_to_string(&path)?).ok_or_else(|| {
                AppError::InvalidInput(format!("« {target} » ne dit pas quelle application lancer"))
            })?;
            let mut exec = entry.exec.into_iter();
            let program = PathBuf::from(exec.next().unwrap_or_default());
            let mut all_args: Vec<String> = exec.collect();
            all_args.extend(split_args(args.as_deref().unwrap_or_default()));
            spawn(&target, &program, all_args, None)
        }
    }
}

/// Renvoie l'icône d'un programme, d'un fichier ou d'un dossier, en image PNG
/// (`data:image/png;base64,…`), ou `None` si la cible n'a pas d'icône (URL, fichier absent).
#[tauri::command]
pub async fn launcher_icon(target: String) -> AppResult<Option<String>> {
    let (path, is_desktop) = match Target::parse(&target)? {
        Target::Url(_) => return Ok(None),
        Target::Program(path) | Target::Path(path) => (path, false),
        Target::Desktop(path) => (path, true),
    };
    if !path.exists() {
        return Ok(None);
    }
    tauri::async_runtime::spawn_blocking(move || {
        if is_desktop {
            Ok(desktop::icon_data_url(&path))
        } else {
            icon::extract_data_url(&path)
        }
    })
    .await
    .map_err(|e| AppError::Other(format!("extraction de l'icône interrompue : {e}")))?
}

/// Taille maximale d'une icône personnalisée : elle est gardée dans `config.json`.
pub const MAX_IMAGE_BYTES: u64 = 512 * 1024;

/// Type MIME d'une image, d'après son extension.
fn image_mime(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        _ => return None,
    })
}

/// Lit une image choisie comme icône personnalisée et la renvoie en `data:` URL.
/// L'image est copiée dans la configuration : déplacer le fichier ensuite ne casse rien.
#[tauri::command]
pub fn launcher_read_image(path: String) -> AppResult<String> {
    let path = PathBuf::from(path);
    let mime = image_mime(&path).ok_or_else(|| {
        AppError::InvalidInput("choisissez une image PNG, JPEG, GIF, WebP, SVG, ICO ou BMP".into())
    })?;
    ensure_exists(&path)?;
    if std::fs::metadata(&path)?.len() > MAX_IMAGE_BYTES {
        return Err(AppError::InvalidInput(
            "l'image dépasse 512 Ko, choisissez-en une plus petite".into(),
        ));
    }
    let bytes = std::fs::read(&path)?;
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
    Ok(format!("data:{mime};base64,{encoded}"))
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

    #[cfg(target_os = "linux")]
    #[test]
    fn desktop_files_and_executables_are_programs_on_linux() {
        assert_eq!(
            Target::parse("/usr/share/applications/firefox.desktop").unwrap(),
            Target::Desktop(PathBuf::from("/usr/share/applications/firefox.desktop"))
        );
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("outil");
        std::fs::write(&script, "#!/bin/sh\n").unwrap();
        assert!(!is_executable_file(&script));
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(is_executable_file(&script));
        assert!(!is_executable_file(dir.path()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_desktop_file_without_command_gives_a_clear_message() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("vide.desktop");
        std::fs::write(&file, "[Desktop Entry]\nName=Vide\n").unwrap();
        let error = launcher_launch(file.display().to_string(), None).unwrap_err();
        assert_eq!(error.code(), "invalid_input");
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
    fn reads_a_small_image_as_data_url() {
        let dir = tempfile::tempdir().unwrap();
        let image = dir.path().join("icone.PNG");
        std::fs::write(&image, [1, 2, 3]).unwrap();
        let url = launcher_read_image(image.display().to_string()).unwrap();
        assert_eq!(url, "data:image/png;base64,AQID");
    }

    #[test]
    fn refuses_other_files_and_big_images() {
        let dir = tempfile::tempdir().unwrap();
        let text = dir.path().join("notes.txt");
        std::fs::write(&text, "bonjour").unwrap();
        let error = launcher_read_image(text.display().to_string()).unwrap_err();
        assert_eq!(error.code(), "invalid_input");

        let big = dir.path().join("grande.png");
        std::fs::write(&big, vec![0; MAX_IMAGE_BYTES as usize + 1]).unwrap();
        let error = launcher_read_image(big.display().to_string()).unwrap_err();
        assert!(error.to_string().contains("512 Ko"));
    }

    #[test]
    fn url_and_missing_file_have_no_icon() {
        let icon = |target: &str| tauri::async_runtime::block_on(launcher_icon(target.into()));
        assert_eq!(icon("https://exemple.fr").unwrap(), None);
        assert_eq!(icon("/pas/la.exe").unwrap(), None);
    }
}
