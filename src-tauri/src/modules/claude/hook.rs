//! Script lancé par Claude Code, et réglage à ajouter à Claude Code.
//!
//! Sous Windows, c'est un script PowerShell ; ailleurs (Linux), un script `sh` qui
//! envoie l'événement avec `curl`.

use std::path::Path;

/// Nom du script, dans le dossier de données de l'application.
#[cfg(windows)]
pub const SCRIPT_NAME: &str = "claude-hook.ps1";
#[cfg(not(windows))]
pub const SCRIPT_NAME: &str = "claude-hook.sh";

/// Emplacement des réglages de Claude Code, tel qu'affiché à l'utilisateur.
#[cfg(windows)]
pub const SETTINGS_FILE: &str = r"%USERPROFILE%\.claude\settings.json";
#[cfg(not(windows))]
pub const SETTINGS_FILE: &str = "~/.claude/settings.json";

/// Script du système en cours.
pub fn script(token: &str, port: u16) -> String {
    if cfg!(windows) {
        powershell_script(token, port)
    } else {
        sh_script(token, port)
    }
}

/// Commande qui lance le script, pour le système en cours.
pub fn command(script: &Path) -> String {
    if cfg!(windows) {
        powershell_command(script)
    } else {
        sh_command(script)
    }
}

/// Script PowerShell, avec la marque UTF-8 au début : sans elle, Windows PowerShell
/// lirait mal les accents des commentaires. Il lit le JSON que Claude Code lui donne, l'envoie au serveur
/// local, et ne bloque jamais Claude Code (3 secondes au plus, erreurs ignorées).
pub fn powershell_script(token: &str, port: u16) -> String {
    format!(
        "\u{feff}# Augmented PC Deck : transmet un événement de Claude Code au tableau de bord.\r\n\
         # Fichier créé par l'application. Il contient le jeton du serveur local :\r\n\
         # ne le partagez pas (Paramètres > Claude > Régénérer le jeton en cas de doute).\r\n\
         $ErrorActionPreference = 'SilentlyContinue'\r\n\
         [Console]::InputEncoding = [System.Text.Encoding]::UTF8\r\n\
         $body = [System.Text.Encoding]::UTF8.GetBytes([Console]::In.ReadToEnd())\r\n\
         Invoke-WebRequest -UseBasicParsing -Method Post -TimeoutSec 3 `\r\n\
         \x20 -Uri 'http://127.0.0.1:{port}{path}' `\r\n\
         \x20 -Headers @{{ Authorization = 'Bearer {token}' }} `\r\n\
         \x20 -ContentType 'application/json; charset=utf-8' -Body $body | Out-Null\r\n\
         exit 0\r\n",
        path = super::server::EVENT_PATH,
    )
}

/// Commande PowerShell qui lance le script.
pub fn powershell_command(script: &Path) -> String {
    format!(
        "powershell -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
        script.display()
    )
}

/// Script `sh` : même rôle que le script PowerShell. `curl` lit le JSON sur l'entrée
/// standard (`--data-binary @-`) ; le jeton passe par un en-tête lu sur un descripteur
/// (`-H @/dev/fd/3`), pour ne pas apparaître dans la liste des processus. Le serveur
/// est local : on ignore un éventuel proxy (`--noproxy`).
pub fn sh_script(token: &str, port: u16) -> String {
    format!(
        "#!/bin/sh\n\
         # Augmented PC Deck : transmet un événement de Claude Code au tableau de bord.\n\
         # Fichier créé par l'application. Il contient le jeton du serveur local :\n\
         # ne le partagez pas (Paramètres > Claude > Régénérer le jeton en cas de doute).\n\
         curl -s -o /dev/null --max-time 3 --noproxy '*' -X POST \\\n\
         \x20 -H 'Content-Type: application/json; charset=utf-8' \\\n\
         \x20 -H @/dev/fd/3 --data-binary @- \\\n\
         \x20 'http://127.0.0.1:{port}{path}' 3<<'EOF'\n\
         Authorization: Bearer {token}\n\
         EOF\n\
         exit 0\n",
        path = super::server::EVENT_PATH,
    )
}

/// Commande `sh` qui lance le script (le chemin est entre apostrophes).
pub fn sh_command(script: &Path) -> String {
    let path = script.display().to_string().replace('\'', r"'\''");
    format!("sh '{path}'")
}

/// Bloc `hooks` à ajouter dans `~/.claude/settings.json`.
pub fn settings_snippet(script: &Path) -> String {
    let hook =
        serde_json::json!([{ "hooks": [{ "type": "command", "command": command(script) }] }]);
    let snippet = serde_json::json!({ "hooks": { "Notification": hook, "Stop": hook } });
    serde_json::to_string_pretty(&snippet).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_powershell_script_posts_to_the_local_server_with_the_token() {
        let script = powershell_script("abc_123", 47821);
        assert!(script.contains("'http://127.0.0.1:47821/claude/event'"));
        assert!(script.contains("Authorization = 'Bearer abc_123'"));
        assert!(script.contains("exit 0"));
        // Retours à la ligne Windows, pour l'éditeur de texte le plus simple.
        assert!(!script.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn the_sh_script_posts_to_the_local_server_with_the_token() {
        let script = sh_script("abc_123", 47821);
        assert!(script.starts_with("#!/bin/sh\n"));
        assert!(script.contains("'http://127.0.0.1:47821/claude/event'"));
        assert!(script.contains("\nAuthorization: Bearer abc_123\nEOF\n"));
        assert!(script.ends_with("exit 0\n"));
        assert!(!script.contains('\r'));
    }

    #[test]
    fn the_powershell_command_runs_the_script() {
        let path = Path::new(r"C:\Users\moi\AppData\Roaming\deck\claude-hook.ps1");
        let command = powershell_command(path);
        assert!(command.starts_with("powershell -NoProfile"));
        assert!(command.ends_with(r#"-File "C:\Users\moi\AppData\Roaming\deck\claude-hook.ps1""#));
    }

    #[test]
    fn the_sh_command_quotes_the_path() {
        let path = Path::new("/home/moi/l'appli/claude-hook.sh");
        assert_eq!(
            sh_command(path),
            r"sh '/home/moi/l'\''appli/claude-hook.sh'"
        );
    }

    #[test]
    fn the_settings_snippet_is_valid_json_for_both_hooks() {
        let path = Path::new("/home/moi/.local/share/deck/claude-hook.sh");
        let snippet: serde_json::Value = serde_json::from_str(&settings_snippet(path)).unwrap();
        for event in ["Notification", "Stop"] {
            let command = snippet["hooks"][event][0]["hooks"][0]["command"]
                .as_str()
                .unwrap();
            assert_eq!(command, super::command(path));
        }
    }
}
