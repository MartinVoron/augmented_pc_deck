//! Script PowerShell lancé par Claude Code, et réglage à ajouter à Claude Code.

use std::path::Path;

/// Nom du script, dans le dossier de données de l'application.
pub const SCRIPT_NAME: &str = "claude-hook.ps1";

/// Contenu du script, avec la marque UTF-8 au début : sans elle, Windows PowerShell
/// lirait mal les accents des commentaires. Il lit le JSON que Claude Code lui donne, l'envoie au serveur
/// local, et ne bloque jamais Claude Code (3 secondes au plus, erreurs ignorées).
pub fn script(token: &str, port: u16) -> String {
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

/// Commande qui lance le script.
pub fn command(script: &Path) -> String {
    format!(
        "powershell -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
        script.display()
    )
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
    fn the_script_posts_to_the_local_server_with_the_token() {
        let script = script("abc_123", 47821);
        assert!(script.contains("'http://127.0.0.1:47821/claude/event'"));
        assert!(script.contains("Authorization = 'Bearer abc_123'"));
        assert!(script.contains("exit 0"));
        // Retours à la ligne Windows, pour l'éditeur de texte le plus simple.
        assert!(!script.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn the_settings_snippet_is_valid_json_for_both_hooks() {
        let path = Path::new(r"C:\Users\moi\AppData\Roaming\deck\claude-hook.ps1");
        let snippet: serde_json::Value = serde_json::from_str(&settings_snippet(path)).unwrap();
        for event in ["Notification", "Stop"] {
            let command = snippet["hooks"][event][0]["hooks"][0]["command"]
                .as_str()
                .unwrap();
            assert!(command.starts_with("powershell -NoProfile"));
            assert!(
                command.ends_with(r#"-File "C:\Users\moi\AppData\Roaming\deck\claude-hook.ps1""#)
            );
        }
    }
}
