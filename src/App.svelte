<script lang="ts">
  import { getAppVersion } from "./lib/modules/app";

  let version = $state("");
  let error = $state("");

  // Vérifie que le front et le Rust communiquent bien.
  getAppVersion()
    .then((v) => (version = v))
    .catch((e: Error) => (error = e.message));
</script>

<main>
  <h1>Augmented PC Deck</h1>
  <p>Le socle de l'application est prêt.</p>
  {#if version}
    <p class="info">Version {version}</p>
  {:else if error}
    <p class="info">Le moteur Tauri n'est pas joignable : {error}</p>
  {/if}
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 1rem;
    text-align: center;
  }

  h1 {
    margin: 0 0 0.5rem;
    font-size: 2rem;
  }

  .info {
    color: #9aa0a6;
    font-size: 0.875rem;
  }
</style>
