<!-- Section « Général » : langue et notifications. -->
<script lang="ts">
  import Toggle from "../../components/Toggle.svelte";
  import { sendNotification } from "../../notifications";
  import { settings, updateSettings } from "./state.svelte";

  let testMessage = $state("");

  async function sendTest() {
    try {
      const sent = await sendNotification({
        title: "Augmented PC Deck",
        body: "Les notifications fonctionnent.",
        sound: "default",
      });
      testMessage = sent ? "Notification envoyée." : "Les notifications sont coupées.";
    } catch (e) {
      testMessage = (e as Error).message;
    }
  }
</script>

{#if settings.config}
  <h3>Langue</h3>
  <label class="field">
    <span>Langue de l'interface</span>
    <select
      value={settings.config.general.language}
      onchange={(event) => {
        const language = event.currentTarget.value;
        void updateSettings((config) => (config.general.language = language));
      }}
    >
      <option value="fr">Français</option>
    </select>
  </label>

  <h3>Notifications</h3>
  <Toggle
    label="Afficher les notifications Windows"
    checked={settings.config.general.notificationsEnabled}
    onchange={(enabled) =>
      updateSettings((config) => (config.general.notificationsEnabled = enabled))}
  />
  <div class="row">
    <button type="button" onclick={sendTest}>Envoyer une notification d'essai</button>
    {#if testMessage}<span class="hint" role="status">{testMessage}</span>{/if}
  </div>
{/if}

<style>
  h3 {
    margin: 1.25rem 0 0.25rem;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
  }

  h3:first-child {
    margin-top: 0;
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.5rem 0;
  }

  select,
  button {
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: var(--color-surface-hover);
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0;
  }

  .hint {
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }
</style>
