<!--
  Tuile « Gmail » : nombre de non lus et derniers messages (expéditeur, objet, heure).
  Rafraîchie toutes les quelques minutes (réglable), en pause quand la fenêtre est
  masquée. Un clic ouvre le message dans Gmail. En 1×1, seul le nombre s'affiche :
  un clic ouvre la boîte de réception.
-->
<script lang="ts">
  import { RefreshCw } from "@lucide/svelte";
  import { clampRefreshMinutes, getUnread, inboxUrl, type Inbox } from ".";
  import type { TileConfig } from "../../grid/defaultLayout";
  import { untrack } from "svelte";
  import { poll } from "../../polling";
  import { openInBrowser, timeAgo } from "../github";
  import { settings } from "../settings/state.svelte";

  let { tile }: { tile: TileConfig } = $props();

  let inbox = $state<Inbox | null>(null);
  let error = $state("");
  let notConnected = $state(false);
  let loading = $state(false);
  let now = $state(Date.now());

  let compact = $derived(tile.size === "1x1");
  let refreshMs = $derived(clampRefreshMinutes(settings.config?.gmail?.refreshMinutes) * 60_000);
  let notify = $derived(settings.config?.gmail?.notifyNew ?? true);

  async function refresh() {
    loading = true;
    try {
      inbox = await getUnread(notify);
      error = "";
      notConnected = false;
    } catch (e) {
      const message = (e as Error).message;
      notConnected = message.includes("aucun compte Gmail");
      error = message;
    } finally {
      loading = false;
      now = Date.now();
    }
  }

  // Relance le rafraîchissement régulier si l'intervalle change.
  $effect(() => {
    const ms = refreshMs;
    return untrack(() => poll(refresh, ms));
  });

  $effect(() => {
    const interval = setInterval(() => (now = Date.now()), 30_000);
    return () => clearInterval(interval);
  });

  function open(url: string) {
    openInBrowser(url).catch((e: Error) => (error = e.message));
  }

  function received(ms: number): string {
    return timeAgo(new Date(ms).toISOString(), now);
  }
</script>

<div class="gmail" data-size={tile.size}>
  {#if notConnected}
    <p class="hint">Connectez votre compte dans Paramètres › Gmail.</p>
  {:else if !inbox && error}
    <p class="error" role="alert">{error}</p>
  {:else if !inbox}
    <p class="hint">Chargement…</p>
  {:else}
    <header>
      <button
        type="button"
        class="count"
        class:attention={inbox.unread > 0}
        aria-label="{inbox.unread} non lus : ouvrir la boîte de réception"
        onclick={() => open(inboxUrl(inbox!.email))}
      >
        <strong>{inbox.unread}</strong>
        <span>{inbox.unread > 1 ? "non lus" : "non lu"}</span>
      </button>
      {#if !compact}
        <span class="email" title={inbox.email}>{inbox.email}</span>
        <button
          type="button"
          class="icon-button"
          class:spinning={loading}
          aria-label="Actualiser"
          disabled={loading}
          onclick={() => void refresh()}
        >
          <RefreshCw size={13} />
        </button>
      {/if}
    </header>

    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}

    {#if !compact}
      {#if inbox.messages.length === 0}
        <p class="hint">Aucun e-mail non lu.</p>
      {:else}
        <ul>
          {#each inbox.messages as message (message.id)}
            <li>
              <button type="button" class="message" onclick={() => open(message.url)}>
                <span class="line">
                  <span class="from">{message.from}</span>
                  <span class="time">{received(message.receivedAt)}</span>
                </span>
                <span class="subject">{message.subject}</span>
                {#if tile.size === "2x2" && message.snippet}
                  <span class="snippet">{message.snippet}</span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  {/if}
</div>

<style>
  .gmail {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    height: 100%;
    min-height: 0;
    font-size: 0.8rem;
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .count {
    display: flex;
    align-items: baseline;
    gap: 0.3rem;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    cursor: pointer;
  }

  .count strong {
    color: var(--color-text);
    font-size: 1.4rem;
    line-height: 1;
  }

  .count.attention strong {
    color: var(--tile-color);
  }

  [data-size="1x1"] header {
    flex: 1;
    justify-content: center;
  }

  [data-size="1x1"] .count {
    flex-direction: column;
    align-items: center;
  }

  [data-size="1x1"] .count strong {
    font-size: 2rem;
  }

  .email {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: var(--color-text-muted);
    font-size: 0.7rem;
    text-align: right;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  ul {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 0.1rem;
    min-height: 0;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .message {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: 0.25rem 0.35rem;
    border: none;
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .message:hover {
    background: var(--color-surface-hover);
  }

  .line {
    display: flex;
    gap: 0.5rem;
    width: 100%;
  }

  .from {
    flex: 1;
    font-weight: 600;
  }

  .time {
    flex: none;
    color: var(--color-text-muted);
    font-size: 0.7rem;
  }

  .from,
  .subject,
  .snippet {
    min-width: 0;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .snippet {
    color: var(--color-text-muted);
    font-size: 0.7rem;
  }

  .icon-button {
    display: inline-flex;
    padding: 0.2rem;
    border: none;
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon-button:hover:not(:disabled) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .spinning :global(svg) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
  }

  .error {
    margin: 0;
    color: var(--color-danger-text);
  }
</style>
