# Connecter Gmail : créer l'identifiant OAuth Google

Augmented PC Deck lit vos e-mails non lus avec l'API Gmail, **en lecture seule**
(droit `gmail.readonly` : l'application ne peut ni envoyer, ni modifier, ni supprimer
un e-mail).

Google demande qu'une application qui accède à Gmail ait son propre identifiant OAuth.
Comme l'application tourne seulement sur votre PC, vous créez cet identifiant vous-même,
une seule fois. Il faut environ 10 minutes, et c'est gratuit.

## 1. Créer un projet Google Cloud

1. Ouvrez la [console Google Cloud](https://console.cloud.google.com/) avec votre compte Google.
2. En haut, cliquez sur le sélecteur de projet, puis **Nouveau projet**.
3. Nom : `Augmented PC Deck` (ou ce que vous voulez), puis **Créer**.
4. Vérifiez que ce projet est bien sélectionné en haut de la page.

## 2. Activer l'API Gmail

1. Ouvrez [API Gmail](https://console.cloud.google.com/apis/library/gmail.googleapis.com).
2. Cliquez sur **Activer**.

## 3. Configurer l'écran de consentement

1. Ouvrez [Plateforme Google Auth > Branding](https://console.cloud.google.com/auth/branding)
   (dans d'anciennes versions de la console : **API et services > Écran de consentement OAuth**).
2. Cliquez sur **Commencer** si on vous le propose, puis renseignez :
   - nom de l'application : `Augmented PC Deck` ;
   - adresse e-mail d'assistance : la vôtre ;
   - type d'utilisateurs (audience) : **Externe** ;
   - coordonnées : votre adresse.
3. Dans **Audience**, ajoutez **votre adresse Gmail** comme **utilisateur test**.

Vous n'avez pas besoin de publier l'application ni de la faire valider par Google :
elle reste en mode « Test », réservée aux utilisateurs test que vous avez ajoutés.

## 4. Créer l'identifiant OAuth

1. Ouvrez [Plateforme Google Auth > Clients](https://console.cloud.google.com/auth/clients)
   (ou **API et services > Identifiants > Créer des identifiants > ID client OAuth**).
2. Type d'application : **Application de bureau**. Nom : `Augmented PC Deck`.
3. Cliquez sur **Créer**. Google affiche :
   - l'**ID client**, qui se termine par `.apps.googleusercontent.com` ;
   - le **code secret du client**, qui commence par `GOCSPX-`.

## 5. Se connecter dans l'application

1. Dans Augmented PC Deck : **Paramètres > Gmail**.
2. Collez l'ID client et le code secret, puis cliquez sur **Se connecter avec Google**.
3. Le navigateur s'ouvre sur la page de Google. Choisissez votre compte.
4. Google prévient que l'application n'est pas validée : c'est normal, c'est la vôtre.
   Cliquez sur **Continuer**.
5. Acceptez l'accès en lecture seule. La page affiche « Connexion réussie » :
   vous pouvez fermer l'onglet.

L'ID client, le code secret et l'accès accordé par Google sont rangés dans le
**Gestionnaire d'identification Windows** (entrée `augmented-pc-deck`, compte `gmail`).
Ils ne sont jamais écrits dans `config.json` ni dans le journal.

## Bon à savoir

- **Reconnexion tous les 7 jours.** Tant que l'application Google reste en mode « Test »,
  Google retire l'accès au bout de 7 jours. La tuile Gmail affiche alors
  « reconnectez-vous » : dans Paramètres > Gmail, cliquez sur **Se reconnecter**
  (l'ID client et le code secret enregistrés sont réutilisés).
  Pour éviter cela, il faudrait publier l'application et la faire valider par Google,
  une démarche longue prévue pour les applications distribuées au public.
- **Se déconnecter** (Paramètres > Gmail) retire aussi l'accès chez Google.
- Vous pouvez à tout moment voir ou retirer l'accès depuis
  [votre compte Google > Sécurité > Applications tierces](https://myaccount.google.com/connections).
