<!--
  Paramètres Gmail : connexion OAuth 2.0 en lecture seule.
  L'ID client et le code secret viennent de la console Google Cloud (voir le guide) ;
  ils partent au Rust, qui ouvre la page de connexion de Google dans le navigateur
  et range tout dans le magasin de jetons du système.
-->
<script lang="ts">
  import { BookOpen, ExternalLink, LogOut, RefreshCw } from "@lucide/svelte";
  import Toggle from "../../components/Toggle.svelte";
  import { openInBrowser } from "../github";
  import { settings, updateSettings } from "../settings/state.svelte";
  import {
    clampRefreshMinutes,
    connectGmail,
    CREDENTIALS_URL,
    disconnectGmail,
    getGmailAccount,
    GUIDE_URL,
    MAX_REFRESH_MINUTES,
    MIN_REFRESH_MINUTES,
    type GmailAccount,
  } from ".";

  /** `undefined` : vérification en cours ; `null` : aucun compte connecté. */
  let account = $state<GmailAccount | null | undefined>(undefined);
  let clientId = $state("");
  let clientSecret = $state("");
  let waiting = $state(false);
  let confirming = $state(false);
  let error = $state("");

  let canConnect = $derived(clientId.trim() !== "" && clientSecret.trim() !== "" && !waiting);

  async function refresh() {
    try {
      account = await getGmailAccount();
    } catch (e) {
      account = null;
      error = (e as Error).message;
    }
  }

  async function connect(event?: SubmitEvent) {
    event?.preventDefault();
    if (event && !canConnect) return;
    waiting = true;
    error = "";
    try {
      account = await connectGmail(clientId, clientSecret);
      clientId = "";
      clientSecret = "";
    } catch (e) {
      error = (e as Error).message;
    } finally {
      waiting = false;
    }
  }

  async function disconnect() {
    confirming = false;
    try {
      await disconnectGmail();
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

  let gmail = $derived({
    refreshMinutes: clampRefreshMinutes(settings.config?.gmail?.refreshMinutes),
    notifyNew: settings.config?.gmail?.notifyNew ?? true,
  });

  function setGmail(change: Partial<typeof gmail>) {
    void updateSettings((config) => (config.gmail = { ...gmail, ...change }));
  }
</script>

<h3>Connexion</h3>

{#if account === undefined}
  <p class="hint">Vérification du compte…</p>
{:else if account}
  <div class="account">
    <div class="who">
      <strong>{account.email}</strong>
      <span class="hint">Connecté en lecture seule</span>
    </div>
    {#if confirming}
      <span class="actions">
        <button type="button" class="secondary" onclick={() => (confirming = false)}>
          Annuler
        </button>
        <button type="button" class="danger" onclick={disconnect}>Confirmer</button>
      </span>
    {:else}
      <span class="actions">
        <button type="button" disabled={waiting} onclick={() => connect()}>
          <RefreshCw size={14} /> Se reconnecter
        </button>
        <button type="button" class="danger" onclick={() => (confirming = true)}>
          <LogOut size={14} /> Se déconnecter
        </button>
      </span>
    {/if}
  </div>
  {#if waiting}
    <p class="hint" role="status">
      Terminez la connexion dans le navigateur qui vient de s'ouvrir…
    </p>
  {/if}
{:else}
  <form onsubmit={connect}>
    <p class="hint">
      Gmail demande votre propre identifiant OAuth, à créer une seule fois dans la console Google
      Cloud (type « Application de bureau »). L'application ne pourra que lire vos e-mails&nbsp;: ni
      envoyer, ni supprimer.
    </p>
    <div class="links">
      <button type="button" class="link" onclick={() => open(GUIDE_URL)}>
        <BookOpen size={14} /> Guide pas à pas
      </button>
      <button type="button" class="link" onclick={() => open(CREDENTIALS_URL)}>
        <ExternalLink size={14} /> Console Google Cloud
      </button>
    </div>
    <label>
      <span>ID client</span>
      <input
        autocomplete="off"
        spellcheck="false"
        placeholder="…apps.googleusercontent.com"
        bind:value={clientId}
      />
    </label>
    <label>
      <span>Code secret du client</span>
      <input
        type="password"
        autocomplete="off"
        spellcheck="false"
        placeholder="GOCSPX-…"
        bind:value={clientSecret}
      />
    </label>
    {#if waiting}
      <p class="hint" role="status">
        Terminez la connexion dans le navigateur qui vient de s'ouvrir…
      </p>
    {/if}
    <div class="actions">
      <button type="submit" class="primary" disabled={!canConnect}>
        {waiting ? "En attente de Google…" : "Se connecter avec Google"}
      </button>
    </div>
  </form>
{/if}

{#if error}<p class="error" role="alert">{error}</p>{/if}

<h3 class="spaced">E-mails non lus</h3>
<Toggle
  label="Notification pour chaque nouvel e-mail"
  checked={gmail.notifyNew}
  onchange={(notifyNew) => setGmail({ notifyNew })}
/>
<label class="inline">
  <span>Actualiser toutes les</span>
  <input
    type="number"
    min={MIN_REFRESH_MINUTES}
    max={MAX_REFRESH_MINUTES}
    value={gmail.refreshMinutes}
    onchange={(event) =>
      setGmail({ refreshMinutes: clampRefreshMinutes(event.currentTarget.valueAsNumber) })}
  />
  <span>minutes</span>
</label>

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
    margin-top: 0.4rem;
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

  .who {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .links,
  .actions {
    display: flex;
    gap: 1rem;
  }

  .actions {
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
    border: none;
    padding: 0;
    color: var(--color-accent);
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
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
