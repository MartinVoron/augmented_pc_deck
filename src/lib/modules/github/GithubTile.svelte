<!--
  Tuile « GitHub » : PR à relire, mes PR ouvertes, issues assignées et notifications
  non lues. Rafraîchie toutes les quelques minutes (réglable dans les paramètres),
  en pause quand la fenêtre est masquée. Un clic ouvre l'élément dans le navigateur.
  En 1×1, seuls les compteurs s'affichent : un clic ouvre la page GitHub de la liste.
-->
<script lang="ts">
  import { ExternalLink, RefreshCw } from "@lucide/svelte";
  import {
    clampRefreshMinutes,
    getGithubActivity,
    openInBrowser,
    reasonLabel,
    SECTIONS,
    timeAgo,
    type GithubActivity,
    type SectionKey,
  } from ".";
  import type { TileConfig } from "../../grid/defaultLayout";
  import { untrack } from "svelte";
  import { poll } from "../../polling";
  import { settings } from "../settings/state.svelte";

  let { tile }: { tile: TileConfig } = $props();

  /** Pages de GitHub qui montrent chaque liste en entier. */
  const PAGES: Record<SectionKey, string> = {
    reviewRequests: "https://github.com/pulls/review-requested",
    myPullRequests: "https://github.com/pulls",
    assignedIssues: "https://github.com/issues/assigned",
    notifications: "https://github.com/notifications",
  };

  let activity = $state<GithubActivity | null>(null);
  let error = $state("");
  let notConnected = $state(false);
  let loading = $state(false);
  let updatedAt = $state<number | null>(null);
  let now = $state(Date.now());
  let current = $state<SectionKey>("reviewRequests");

  let compact = $derived(tile.size === "1x1");
  let section = $derived(activity?.[current]);
  let refreshMs = $derived(clampRefreshMinutes(settings.config?.github?.refreshMinutes) * 60_000);

  async function refresh() {
    loading = true;
    try {
      activity = await getGithubActivity();
      error = "";
      notConnected = false;
      updatedAt = Date.now();
    } catch (e) {
      const message = (e as Error).message;
      notConnected = message.includes("aucun compte GitHub");
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

  // Met à jour « il y a … » sans refaire de requête.
  $effect(() => {
    const interval = setInterval(() => (now = Date.now()), 30_000);
    return () => clearInterval(interval);
  });

  function open(url: string) {
    openInBrowser(url).catch((e: Error) => (error = e.message));
  }

  function choose(key: SectionKey) {
    if (compact) open(PAGES[key]);
    else current = key;
  }
</script>

<div class="github" data-size={tile.size}>
  {#if notConnected}
    <p class="hint">Connectez votre compte dans Paramètres › GitHub.</p>
  {:else if !activity && error}
    <p class="error" role="alert">{error}</p>
  {:else if !activity}
    <p class="hint">Chargement…</p>
  {:else}
    <div class="counters" role="group" aria-label="Listes GitHub">
      {#each SECTIONS as item (item.key)}
        {@const s = activity[item.key]}
        <button
          type="button"
          class="counter"
          class:active={!compact && item.key === current}
          class:attention={s.total > 0 && item.key !== "myPullRequests"}
          aria-pressed={compact ? undefined : item.key === current}
          title={s.error ?? item.label}
          aria-label="{item.label} : {s.error ? 'erreur' : s.total}"
          onclick={() => choose(item.key)}
        >
          <span class="count">{s.error ? "!" : s.total}</span>
          <span class="label">{item.short}</span>
        </button>
      {/each}
    </div>

    {#if !compact && section}
      {#if section.error}
        <p class="error" role="alert">{section.error}</p>
      {:else if section.items.length === 0}
        <p class="hint">Rien en attente.</p>
      {:else}
        <ul>
          {#each section.items as item (item.url + item.title)}
            <li>
              <button type="button" class="item" onclick={() => open(item.url)}>
                <span class="title">{item.title}</span>
                <span class="meta">
                  {item.repo}{item.number ? `#${item.number}` : ""}
                  {#if item.reason}· {reasonLabel(item.reason)}{/if}
                  · {timeAgo(item.updatedAt, now)}
                </span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}

    <footer hidden={compact}>
      {#if error}
        <span class="stale" title={error}>Hors ligne</span>
      {:else if updatedAt}
        <span>Mis à jour {timeAgo(new Date(updatedAt).toISOString(), now)}</span>
      {/if}
      <span class="tools">
        {#if !compact}
          <button
            type="button"
            class="icon-button"
            aria-label="Ouvrir sur GitHub"
            onclick={() => open(PAGES[current])}
          >
            <ExternalLink size={13} />
          </button>
        {/if}
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
      </span>
    </footer>
  {/if}
</div>

<style>
  .github {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    height: 100%;
    min-height: 0;
    font-size: 0.8rem;
  }

  .counters {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.3rem;
  }

  [data-size="1x1"] .counters {
    flex: 1;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.2rem;
  }

  [data-size="1x1"] .counter {
    padding: 0.1rem;
  }

  [data-size="1x1"] .count {
    font-size: 1rem;
  }

  .counter {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.05rem;
    min-width: 0;
    padding: 0.3rem 0.2rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: var(--color-bg);
    color: var(--color-text-muted);
    font: inherit;
    cursor: pointer;
  }

  .counter:hover {
    background: var(--color-surface-hover);
  }

  .counter.active {
    border-color: var(--tile-color);
    color: var(--color-text);
  }

  .count {
    color: var(--color-text);
    font-size: 1.15rem;
    font-weight: 600;
    line-height: 1.1;
  }

  .attention .count {
    color: var(--tile-color);
  }

  .label {
    max-width: 100%;
    overflow: hidden;
    font-size: 0.7rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  ul {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 0.15rem;
    min-height: 0;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .item {
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

  .item:hover {
    background: var(--color-surface-hover);
  }

  .title,
  .meta {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    color: var(--color-text-muted);
    font-size: 0.7rem;
  }

  footer[hidden] {
    display: none;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: auto;
    color: var(--color-text-muted);
    font-size: 0.7rem;
  }

  .tools {
    display: flex;
    gap: 0.15rem;
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

  .stale {
    color: var(--color-warning);
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
  }

  .error {
    margin: 0;
    color: #f1c4c4;
  }
</style>
