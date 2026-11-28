//! Fichiers `.desktop` (Linux) : la description d'une application installée, telle que
//! le menu des applications l'affiche (`/usr/share/applications/firefox.desktop`…).
//!
//! On lit la commande à lancer (`Exec`) et le nom de l'icône (`Icon`), et on cherche
//! cette icône dans les thèmes d'icônes du système.

use std::path::{Path, PathBuf};

use super::split_args;

/// Ce qui nous sert dans un fichier `.desktop`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    /// Programme et arguments, sans les codes `%f`, `%U`…
    pub exec: Vec<String>,
    /// Nom d'icône du thème (`firefox`) ou chemin d'une image.
    pub icon: Option<String>,
}

/// Lit le groupe `[Desktop Entry]`. Renvoie `None` s'il n'y a pas de commande.
pub fn parse(text: &str) -> Option<DesktopEntry> {
    let mut in_entry = false;
    let mut exec = None;
    let mut icon = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "Exec" => exec = Some(value.trim().to_string()),
            "Icon" => icon = Some(value.trim().to_string()).filter(|i| !i.is_empty()),
            _ => {}
        }
    }
    let exec = exec_args(&exec?);
    (!exec.is_empty()).then_some(DesktopEntry { exec, icon })
}

/// Découpe la ligne `Exec` et retire les codes de champ (`%f`, `%U`, `%i`…), qui
/// demandent des fichiers que nous n'avons pas. `%%` devient `%`.
fn exec_args(exec: &str) -> Vec<String> {
    split_args(exec)
        .into_iter()
        .filter(|arg| !(arg.len() == 2 && arg.starts_with('%') && arg != "%%"))
        .map(|arg| arg.replace("%%", "%"))
        .collect()
}

/// Dossiers où chercher les icônes, du plus personnel au plus général.
fn icon_dirs() -> Vec<PathBuf> {
    let mut data_dirs = Vec::new();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        data_dirs.push(home.join(".local/share"));
    }
    let system = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    data_dirs.extend(system.split(':').map(PathBuf::from));
    data_dirs.push("/var/lib/flatpak/exports/share".into());

    let mut dirs = Vec::new();
    for data in &data_dirs {
        for size in ["64x64", "48x48", "128x128", "96x96", "256x256", "32x32"] {
            dirs.push(data.join("icons/hicolor").join(size).join("apps"));
        }
        dirs.push(data.join("icons/hicolor/scalable/apps"));
    }
    for data in &data_dirs {
        dirs.push(data.join("pixmaps"));
    }
    dirs
}

/// Type MIME des images qu'une `<img>` sait afficher.
fn icon_mime(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

/// Fichier d'image d'une icône : chemin donné tel quel, ou recherche dans les thèmes.
pub fn find_icon(icon: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let path = Path::new(icon);
    if path.is_absolute() {
        return (icon_mime(path).is_some() && path.is_file()).then(|| path.to_path_buf());
    }
    dirs.iter()
        .flat_map(|dir| ["png", "svg"].map(|ext| dir.join(format!("{icon}.{ext}"))))
        .find(|candidate| candidate.is_file())
}

/// Taille maximale d'une icône lue.
const MAX_ICON_BYTES: u64 = 1024 * 1024;

/// Icône d'un fichier `.desktop`, en `data:` URL.
pub fn icon_data_url(desktop_file: &Path) -> Option<String> {
    let entry = parse(&std::fs::read_to_string(desktop_file).ok()?)?;
    let path = find_icon(entry.icon.as_deref()?, &icon_dirs())?;
    let mime = icon_mime(&path)?;
    if std::fs::metadata(&path).ok()?.len() > MAX_ICON_BYTES {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
    Some(format!("data:{mime};base64,{encoded}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIREFOX: &str = "\
[Desktop Entry]
Name=Firefox
# Un commentaire
Exec=firefox --new-window %u
Icon=firefox

[Desktop Action new-private-window]
Exec=firefox --private-window %u
";

    #[test]
    fn reads_the_main_command_and_icon() {
        assert_eq!(
            parse(FIREFOX),
            Some(DesktopEntry {
                exec: vec!["firefox".into(), "--new-window".into()],
                icon: Some("firefox".into()),
            })
        );
    }

    #[test]
    fn keeps_quoted_arguments_and_literal_percent() {
        let entry =
            parse("[Desktop Entry]\nExec=\"/opt/Mon Appli/appli\" --taux 50%% %F\n").unwrap();
        assert_eq!(entry.exec, vec!["/opt/Mon Appli/appli", "--taux", "50%"]);
        assert_eq!(entry.icon, None);
    }

    #[test]
    fn a_file_without_command_is_refused() {
        assert_eq!(parse("[Desktop Entry]\nName=Rien\n"), None);
        assert_eq!(parse("[Autre]\nExec=ls\n"), None);
    }

    #[test]
    fn finds_an_icon_in_the_theme_folders() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("48x48/apps");
        std::fs::create_dir_all(&apps).unwrap();
        std::fs::write(apps.join("jeu.svg"), "<svg/>").unwrap();
        let dirs = vec![dir.path().join("64x64/apps"), apps.clone()];
        assert_eq!(find_icon("jeu", &dirs), Some(apps.join("jeu.svg")));
        assert_eq!(find_icon("absent", &dirs), None);

        let absolute = apps.join("jeu.svg");
        assert_eq!(
            find_icon(&absolute.display().to_string(), &[]),
            Some(absolute)
        );
    }
}
