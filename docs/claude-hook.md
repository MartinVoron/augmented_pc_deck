# Recevoir les notifications de Claude Code

Augmented PC Deck peut vous prévenir quand **Claude Code attend une réponse** (une
question ou une autorisation) ou qu'il **a fini une tâche**, même si son terminal est
caché derrière d'autres fenêtres.

## Comment ça marche

Claude Code sait lancer une commande à certains moments : ce sont les
[hooks](https://docs.claude.com/en/docs/claude-code/hooks). Deux hooks sont utilisés :

| Hook           | Quand                                       | Dans la tuile          |
| -------------- | ------------------------------------------- | ---------------------- |
| `Notification` | Claude attend une réponse ou une permission | « attend une réponse » |
| `Stop`         | Claude a fini de répondre                   | « terminé »            |

À chaque fois, Claude Code lance un petit script PowerShell, créé par l'application.
Le script envoie l'événement à un serveur local de l'application :

- il n'écoute que sur `127.0.0.1:47821`, donc aucun autre ordinateur ne peut l'atteindre ;
- il refuse toute requête qui n'a pas le **jeton** de l'application. Ce jeton est créé
  au premier lancement et rangé dans le Gestionnaire d'identification Windows.

Le script ne bloque jamais Claude Code : si l'application est fermée, il abandonne en
silence au bout de 3 secondes au plus.

## Mise en place (une seule fois)

1. Dans Augmented PC Deck : **Paramètres > Claude**.
2. Vérifiez que le serveur local est « À l'écoute ».
3. Cliquez sur **Créer le script**. Il est écrit dans le dossier de l'application
   (`%APPDATA%\fr.martinvoron.augmented-pc-deck\claude-hook.ps1`, chemin affiché dans
   les paramètres).
4. Cliquez sur **Copier le bloc**, puis ouvrez le fichier des réglages de Claude Code :
   `%USERPROFILE%\.claude\settings.json` (créez-le s'il n'existe pas).
5. Collez le bloc :
   - si le fichier est vide ou n'existe pas, collez le bloc tel quel ;
   - s'il contient déjà des réglages, ajoutez la partie `"hooks"` au même niveau que
     les autres (attention aux virgules) ;
   - s'il contient déjà une partie `"hooks"`, ajoutez seulement `"Notification"` et
     `"Stop"` dedans.
6. Redémarrez Claude Code. Dans Claude Code, la commande `/hooks` liste les hooks actifs.

Le bloc ressemble à ceci (le chemin du script est le vôtre) :

```json
{
  "hooks": {
    "Notification": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "powershell -NoProfile -ExecutionPolicy Bypass -File \"C:\\Users\\moi\\AppData\\Roaming\\fr.martinvoron.augmented-pc-deck\\claude-hook.ps1\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "powershell -NoProfile -ExecutionPolicy Bypass -File \"C:\\Users\\moi\\AppData\\Roaming\\fr.martinvoron.augmented-pc-deck\\claude-hook.ps1\""
          }
        ]
      }
    ]
  }
}
```

## Vérifier

Demandez quelque chose à Claude Code. Quand il a fini, un événement « terminé »
apparaît dans la tuile Claude.

Pour tester sans Claude Code, dans PowerShell (remplacez le chemin par celui affiché
dans les paramètres) :

```powershell
'{"hook_event_name":"Notification","cwd":"C:\\dev\\essai","message":"Test"}' |
  powershell -NoProfile -ExecutionPolicy Bypass -File "$env:APPDATA\fr.martinvoron.augmented-pc-deck\claude-hook.ps1"
```

## Sécurité

- Le script contient le jeton du serveur local. Ce jeton permet seulement d'afficher des
  notifications dans l'application : il ne donne accès à rien d'autre.
- Si vous pensez qu'il a été copié, cliquez sur **Régénérer le jeton** : le script est
  réécrit et l'ancien jeton ne marche plus.
- Le port `47821` doit être libre. Si une autre application l'utilise, les paramètres
  l'indiquent.
