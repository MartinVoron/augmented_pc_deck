<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let version = $state("");
  let erreur = $state("");

  // Vérifie que le front et le Rust communiquent bien.
  invoke<string>("app_version")
    .then((v) => (version = v))
    .catch((e) => (erreur = String(e)));
</script>

<main>
  <h1>Augmented PC Deck</h1>
  <p>Le socle de l'application est prêt.</p>
  {#if version}
    <p class="info">Version {version}</p>
  {:else if erreur}
    <p class="info">Le moteur Tauri n'est pas joignable : {erreur}</p>
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
