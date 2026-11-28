<!--
  Protège une tuile : si son contenu plante, l'erreur est écrite dans le journal
  et la tuile affiche un message clair, sans faire planter le reste de l'application.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { logError } from "../logging";

  interface Props {
    /** Nom de la tuile, affiché et écrit dans le journal. */
    name: string;
    children: Snippet;
  }

  let { name, children }: Props = $props();

  function onerror(error: unknown) {
    void logError(`tuile ${name}`, error);
  }
</script>

<svelte:boundary {onerror}>
  {@render children()}

  {#snippet failed(error, reset)}
    <div class="tile-error" role="alert">
      <p class="title">{name} ne répond plus</p>
      <p class="detail">{error instanceof Error ? error.message : String(error)}</p>
      <button type="button" onclick={reset}>Réessayer</button>
    </div>
  {/snippet}
</svelte:boundary>

<style>
  .tile-error {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem;
    border: 1px solid #5c2b2b;
    border-radius: 0.5rem;
    background: var(--color-danger-bg);
    color: var(--color-danger-text);
  }

  .title {
    margin: 0;
    font-weight: 600;
  }

  .detail {
    margin: 0;
    font-size: 0.875rem;
    color: #d9a3a3;
  }

  button {
    align-self: flex-start;
    margin-top: 0.25rem;
    padding: 0.25rem 0.75rem;
    border: 1px solid #8a4a4a;
    border-radius: 0.375rem;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
</style>
