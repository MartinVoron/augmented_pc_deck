<!--
  Paramètres GitHub : connexion avec un jeton personnel, compte connecté, déconnexion.
  Le jeton saisi part directement au Rust, qui le vérifie puis le range dans le
  magasin de jetons du système ; l'interface ne le revoit jamais.
-->
<script lang="ts">
  import { ExternalLink, LogOut } from "@lucide/svelte";
  import { settings, updateSettings } from "../settings/state.svelte";
  import {
    clampRefreshMinutes,
    connectGithub,
    disconnectGithub,
    getGithubAccount,
    MAX_REFRESH_MINUTES,
    MIN_REFRESH_MINUTES,
    missingScopes,
    NEW_TOKEN_URL,
    openInBrowser,
    type GithubAccount,
  } from ".";

  /** `undefined` : vérification en cours ; `null` : aucun compte connecté. */
  let account = $state<GithubAccount | null | undefined>(undefined);
  let token = $state("");
  let busy = $state(false);
  let confirming = $state(false);
  let error = $state("");

  let missing = $derived(account ? missingScopes(account) : []);

  async function refresh() {
    try {
      account = await getGithubAccount();
      error = "";
    } catch (e) {
      // Jeton refusé (révoqué, expiré) : on propose d'en saisir un autre.
      account = null;
      error = (e as Error).message;
    }
  }

  async function connect(event: SubmitEvent) {
    event.preventDefault();
    if (!token.trim()) return;
    busy = true;
    try {
      account = await connectGithub(token);
      token = "";
      error = "";
    } catch (e) {
      error = (e as Error).message;
    } finally {
      busy = false;
    }
  }

  async function disconnect() {
    confirming = false;
    try {
      await disconnectGithub();
      account = null;
      error = "";
    } catch (e) {
      error = (e as Error).message;
    }
  }

  function open(url: string) {
    openInBrowser(url).catch((e: Error) => (error = e.message));
  }

  $effect(() => {
    void refresh();
  });

  let refreshMinutes = $derived(clampRefreshMinutes(settings.config?.github?.refreshMinutes));

  function setRefreshMinutes(value: number) {
    const minutes = clampRefreshMinutes(value);
    void updateSettings(
      (config) => (config.github = { ...config.github, refreshMinutes: minutes }),
    );
  }
</script>

<h3>Connexion</h3>

{#if account === undefined}
  <p class="hint">Vérification du compte…</p>
{:else if account}
  <div class="account">
    <img src={account.avatarUrl} alt="" width="40" height="40" />
    <div class="who">
      <strong>{account.name ?? account.login}</strong>
      <span class="hint">Connecté en tant que @{account.login}</span>
    </div>
    {#if confirming}
      <span class="actions">
        <button type="button" class="secondary" onclick={() => (confirming = false)}>
          Annuler
        </button>
        <button type="button" class="danger" onclick={disconnect}>Confirmer</button>
      </span>
    {:else}
      <button type="button" class="danger" onclick={() => (confirming = true)}>
        <LogOut size={14} /> Se déconnecter
      </button>
    {/if}
  </div>
  {#if missing.length > 0}
    <p class="warning" role="status">
      Ce jeton n'a pas les droits {missing.map((s) => `« ${s} »`).join(" et ")} : une partie de l'activité
      ne pourra pas s'afficher.
    </p>
  {/if}
{:else}
  <form onsubmit={connect}>
    <p class="hint">
      Créez un jeton personnel sur GitHub avec les droits « repo » et « notifications », puis
      collez-le ici. Il est gardé dans le coffre du système (Gestionnaire d'identification sous
      Windows, trousseau sous Linux), jamais dans un fichier.
    </p>
    <button type="button" class="link" onclick={() => open(NEW_TOKEN_URL)}>
      <ExternalLink size={14} /> Créer un jeton sur GitHub
    </button>
    <label>
      <span>Jeton personnel</span>
      <input
        type="password"
        autocomplete="off"
        spellcheck="false"
        placeholder="ghp_… ou github_pat_…"
        bind:value={token}
      />
    </label>
    <div class="actions">
      <button type="submit" class="primary" disabled={busy || !token.trim()}>
        {busy ? "Vérification…" : "Se connecter"}
      </button>
    </div>
  </form>
{/if}

{#if error}<p class="error" role="alert">{error}</p>{/if}

<h3 class="spaced">Activité</h3>
<label class="inline">
  <span>Actualiser toutes les</span>
  <input
    type="number"
    min={MIN_REFRESH_MINUTES}
    max={MAX_REFRESH_MINUTES}
    value={refreshMinutes}
    onchange={(event) => setRefreshMinutes(event.currentTarget.valueAsNumber)}
  />
  <span>minutes</span>
</label>
<p class="hint">
  Chaque actualisation fait quatre requêtes, bien en dessous des limites de l'API GitHub.
</p>

<style>
  h3 {
    margin: 0 0 0.5rem;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
  }

  .spaced {
    margin-top: 1.5rem;
  }

  label.inline {
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.4rem;
    color: var(--color-text);
  }

  label.inline input {
    width: 4.5rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }

  input {
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: var(--color-bg);
    color: var(--color-text);
    font: inherit;
  }

  .account {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0;
  }

  .account img {
    border-radius: 50%;
  }

  .who {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  form .actions {
    justify-content: flex-end;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .primary {
    border-color: var(--color-accent);
    background: var(--color-accent);
    color: #fff;
  }

  .danger {
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  .link {
    align-self: flex-start;
    border: none;
    padding: 0;
    color: var(--color-accent);
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }

  .warning {
    margin: 0.25rem 0 0;
    color: var(--color-warning);
    font-size: 0.875rem;
  }

  .error {
    margin: 0.5rem 0 0;
    padding: 0.4rem 0.6rem;
    border-radius: 0.5rem;
    background: #2a1a1a;
    color: #f1c4c4;
    font-size: 0.875rem;
  }
</style>
