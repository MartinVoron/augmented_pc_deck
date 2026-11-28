<!--
  Paramètres Claude : mise en place du hook qui transmet les événements de
  Claude Code au serveur local de l'application.
  1. Créer le script (il contient le jeton du serveur local) ;
  2. copier le bloc de réglages dans ~/.claude/settings.json.
-->
<script lang="ts">
  import { BookOpen, Check, Copy, FileCode, KeyRound } from "@lucide/svelte";
  import { openInBrowser } from "../github";
  import { getClaudeStatus, GUIDE_URL, installHook, regenerateToken, type ClaudeStatus } from ".";

  let status = $state<ClaudeStatus | null>(null);
  let error = $state("");
  let copied = $state(false);
  let confirming = $state(false);
  let message = $state("");

  async function run(action: () => Promise<ClaudeStatus>, done = "") {
    try {
      status = await action();
      error = "";
      message = done;
    } catch (e) {
      error = (e as Error).message;
    }
  }

  async function copy() {
    if (!status) return;
    try {
      await navigator.clipboard.writeText(status.settingsSnippet);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      error = "Copie impossible : sélectionnez le texte et copiez-le à la main.";
    }
  }

  $effect(() => {
    void run(getClaudeStatus);
  });
</script>

<h3>Serveur local</h3>
{#if !status}
  <p class="hint">Vérification…</p>
{:else if status.server.state === "listening"}
  <p class="ok">À l'écoute sur 127.0.0.1:{status.port} (accessible depuis ce PC seulement).</p>
{:else if status.server.state === "failed"}
  <p class="error" role="alert">
    Le serveur n'a pas démarré : {status.server.error}. Fermez les autres copies de l'application,
    puis relancez-la.
  </p>
{:else}
  <p class="hint">Démarrage…</p>
{/if}

{#if status}
  <h3 class="spaced">1. Script du hook</h3>
  <p class="hint">
    Claude Code lance ce petit script quand il attend une réponse ou qu'il a fini. Il contient le
    jeton qui protège le serveur local.
  </p>
  <div class="row">
    <button type="button" class="primary" onclick={() => run(installHook, "Script créé.")}>
      <FileCode size={14} />
      {status.scriptInstalled ? "Recréer le script" : "Créer le script"}
    </button>
    {#if status.scriptInstalled}
      <code class="path" title={status.scriptPath}>{status.scriptPath}</code>
    {/if}
  </div>

  <h3 class="spaced">2. Réglages de Claude Code</h3>
  <p class="hint">
    Ajoutez ce bloc dans <code>{status.settingsFile}</code> (s'il y a déjà une partie « hooks », ajoutez
    seulement « Notification » et « Stop » dedans), puis redémarrez Claude Code.
  </p>
  <pre>{status.settingsSnippet}</pre>
  <div class="row">
    <button type="button" onclick={copy} disabled={!status.scriptInstalled}>
      {#if copied}<Check size={14} /> Copié{:else}<Copy size={14} /> Copier le bloc{/if}
    </button>
    <button
      type="button"
      class="link"
      onclick={() => openInBrowser(GUIDE_URL).catch((e: Error) => (error = e.message))}
    >
      <BookOpen size={14} /> Guide pas à pas
    </button>
  </div>

  <h3 class="spaced">Jeton</h3>
  <div class="row">
    {#if confirming}
      <span class="hint">L'ancien jeton ne marchera plus.</span>
      <button type="button" onclick={() => (confirming = false)}>Annuler</button>
      <button
        type="button"
        class="danger"
        onclick={() => {
          confirming = false;
          void run(regenerateToken, "Nouveau jeton créé.");
        }}
      >
        Confirmer
      </button>
    {:else}
      <button type="button" onclick={() => (confirming = true)}>
        <KeyRound size={14} /> Régénérer le jeton
      </button>
    {/if}
  </div>
{/if}

{#if message}<p class="hint" role="status">{message}</p>{/if}
{#if error}<p class="error" role="alert">{error}</p>{/if}

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

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  pre {
    max-height: 14rem;
    margin: 0.5rem 0 0;
    padding: 0.6rem 0.75rem;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: var(--color-bg);
    font-size: 0.75rem;
    user-select: text;
  }

  code {
    font-size: 0.8rem;
  }

  .path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: var(--color-text-muted);
    text-overflow: ellipsis;
    white-space: nowrap;
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
    color: var(--color-accent);
  }

  .hint,
  .ok {
    margin: 0;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }

  .ok {
    color: var(--color-success);
  }

  .error {
    margin: 0.5rem 0 0;
    padding: 0.4rem 0.6rem;
    border-radius: 0.5rem;
    background: var(--color-danger-bg);
    color: var(--color-danger-text);
    font-size: 0.875rem;
  }
</style>
