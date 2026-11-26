<!--
  Jeton de connexion d'un service : indique s'il est enregistré et permet de le supprimer.
  La connexion elle-même (saisie du jeton) arrive avec chaque module.
-->
<script lang="ts">
  import { deleteToken, getTokenStatus, type TokenKind } from "../../tokens";

  interface Props {
    kind: TokenKind;
    service: string;
  }

  let { kind, service }: Props = $props();

  let saved = $state<boolean | null>(null);
  let confirming = $state(false);
  let message = $state("");

  async function refresh() {
    try {
      saved = (await getTokenStatus())[kind];
    } catch (e) {
      message = (e as Error).message;
    }
  }

  async function remove() {
    confirming = false;
    try {
      await deleteToken(kind);
      message = "Jeton supprimé.";
      await refresh();
    } catch (e) {
      message = (e as Error).message;
    }
  }

  $effect(() => {
    void refresh();
  });
</script>

<h3>Connexion</h3>
<div class="row">
  <span>
    {#if saved === null}
      Vérification…
    {:else if saved}
      Un jeton {service} est enregistré dans le Gestionnaire d'identification Windows.
    {:else}
      Aucun jeton {service} enregistré.
    {/if}
  </span>
  {#if saved && !confirming}
    <button type="button" onclick={() => (confirming = true)}>Supprimer le jeton</button>
  {:else if saved}
    <span class="actions">
      <button type="button" class="secondary" onclick={() => (confirming = false)}>Annuler</button>
      <button type="button" onclick={remove}>Confirmer la suppression</button>
    </span>
  {/if}
</div>
{#if message}<p class="hint" role="status">{message}</p>{/if}

<style>
  h3 {
    margin: 0 0 0.25rem;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.5rem 0;
  }

  button {
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--color-danger);
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-danger);
    font: inherit;
    cursor: pointer;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .secondary {
    border-color: var(--color-border);
    color: var(--color-text);
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }
</style>
