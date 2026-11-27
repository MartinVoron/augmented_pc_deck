<!--
  Écran des paramètres : une section « Général », puis une section par module.
  Chaque changement est enregistré et appliqué tout de suite.
-->
<script lang="ts">
  import { Settings, X } from "@lucide/svelte";
  import type { Component } from "svelte";
  import { KIND_ICONS } from "../../grid/icons";
  import type { TileKind } from "../../grid/defaultLayout";
  import ClaudeSection from "../claude/ClaudeSection.svelte";
  import GithubSection from "../github/GithubSection.svelte";
  import GmailSection from "../gmail/GmailSection.svelte";
  import GeneralSection from "./GeneralSection.svelte";
  import { settings } from "./state.svelte";

  let { onclose }: { onclose: () => void } = $props();

  type SectionId = "general" | TileKind;

  const sections: { id: SectionId; label: string; icon: Component }[] = [
    { id: "general", label: "Général", icon: Settings },
    { id: "launcher", label: "Raccourcis", icon: KIND_ICONS.launcher },
    { id: "processes", label: "Processus", icon: KIND_ICONS.processes },
    { id: "timers", label: "Minuteurs", icon: KIND_ICONS.timers },
    { id: "todos", label: "Tâches", icon: KIND_ICONS.todos },
    { id: "music", label: "Musique", icon: KIND_ICONS.music },
    { id: "github", label: "GitHub", icon: KIND_ICONS.github },
    { id: "gmail", label: "Gmail", icon: KIND_ICONS.gmail },
    { id: "claude", label: "Claude", icon: KIND_ICONS.claude },
  ];

  let current = $state<SectionId>("general");
  let section = $derived(sections.find((s) => s.id === current)!);
</script>

<div class="screen">
  <header>
    <h1>Paramètres</h1>
    <button type="button" class="close" aria-label="Fermer les paramètres" onclick={onclose}>
      <X size={20} />
    </button>
  </header>

  <div class="body">
    <nav aria-label="Sections des paramètres">
      {#each sections as item (item.id)}
        <button
          type="button"
          class:active={item.id === current}
          aria-current={item.id === current ? "page" : undefined}
          onclick={() => (current = item.id)}
        >
          <item.icon size={16} />
          {item.label}
        </button>
      {/each}
    </nav>

    <section aria-labelledby="section-title">
      <h2 id="section-title">{section.label}</h2>
      {#if settings.error}
        <p class="error" role="alert">{settings.error}</p>
      {/if}

      {#if current === "general"}
        <GeneralSection />
      {:else if current === "github"}
        <GithubSection />
      {:else if current === "gmail"}
        <GmailSection />
      {:else if current === "claude"}
        <ClaudeSection />
      {:else}
        <p class="hint">Aucun réglage pour l'instant.</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .screen {
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem var(--tile-gap);
    border-bottom: 1px solid var(--color-border);
  }

  h1 {
    margin: 0;
    font-size: 1.1rem;
  }

  h2 {
    margin: 0 0 1rem;
    font-size: 1.25rem;
  }

  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    width: 12rem;
    flex-shrink: 0;
    padding: 0.75rem;
    border-right: 1px solid var(--color-border);
  }

  nav button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.6rem;
    border: none;
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    font-size: 0.9rem;
    text-align: left;
    cursor: pointer;
  }

  nav button:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  nav button.active {
    background: var(--color-surface);
    color: var(--color-text);
  }

  section {
    flex: 1;
    min-width: 0;
    max-width: 40rem;
    padding: 1rem 1.5rem;
  }

  .close {
    display: inline-flex;
    padding: 0.25rem;
    border: none;
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .close:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .hint {
    color: var(--color-text-muted);
  }

  .error {
    padding: 0.5rem 0.75rem;
    border-radius: 0.5rem;
    background: #2a1a1a;
    color: #f1c4c4;
  }

  /* Fenêtre étroite : la liste des sections passe au-dessus. */
  @media (max-width: 640px) {
    .body {
      flex-direction: column;
    }

    nav {
      flex-direction: row;
      flex-wrap: wrap;
      width: auto;
      border-right: none;
      border-bottom: 1px solid var(--color-border);
    }
  }
</style>
