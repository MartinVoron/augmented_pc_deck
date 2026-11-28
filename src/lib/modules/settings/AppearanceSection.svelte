<!-- Section « Apparence » : thème, mode clair ou sombre et flou des tuiles. -->
<script lang="ts">
  import {
    MODE_LABELS,
    sanitizeAppearance,
    THEME_LABELS,
    type AppearanceSettings,
    type ColorMode,
    type Theme,
  } from "../../appearance";
  import { blurPreview } from "../../appearance/state.svelte";
  import { settings, updateSettings } from "./state.svelte";

  let appearance = $derived(sanitizeAppearance(settings.config?.appearance));
  /** Valeur affichée : celle du curseur en cours de réglage, sinon celle enregistrée. */
  let blur = $derived(blurPreview.value ?? appearance.blur);

  /** Le curseur est lâché : on enregistre, puis on revient à la valeur enregistrée. */
  async function saveBlur(value: number) {
    blurPreview.value = value;
    await change({ blur: value });
    blurPreview.value = null;
  }

  function change(patch: Partial<AppearanceSettings>): Promise<void> {
    return updateSettings(
      (config) => (config.appearance = { ...sanitizeAppearance(config.appearance), ...patch }),
    );
  }
</script>

{#if settings.config}
  <h3>Thème</h3>
  <div class="themes" role="radiogroup" aria-label="Thème">
    {#each Object.entries(THEME_LABELS) as [theme, label] (theme)}
      <label class="theme" class:selected={appearance.theme === theme}>
        <input
          type="radio"
          name="theme"
          value={theme}
          checked={appearance.theme === theme}
          onchange={() => change({ theme: theme as Theme })}
        />
        <span class="preview {theme}" aria-hidden="true">
          <span></span><span></span><span></span>
        </span>
        {label}
      </label>
    {/each}
  </div>

  <label class="field">
    <span>Clair ou sombre</span>
    <select
      value={appearance.mode}
      disabled={appearance.theme === "classic"}
      onchange={(event) => change({ mode: event.currentTarget.value as ColorMode })}
    >
      {#each Object.entries(MODE_LABELS) as [mode, label] (mode)}
        <option value={mode}>{label}</option>
      {/each}
    </select>
  </label>
  {#if appearance.theme === "classic"}
    <p class="hint">Le thème Classique n'existe qu'en sombre.</p>
  {/if}

  <h3>Flou</h3>
  <div class="field">
    <label for="glass-blur">Flou des tuiles</label>
    <span class="slider">
      <input
        id="glass-blur"
        type="range"
        min="0"
        max="100"
        step="5"
        value={blur}
        disabled={appearance.theme !== "glass"}
        aria-valuetext="{blur} %"
        oninput={(event) => (blurPreview.value = Number(event.currentTarget.value))}
        onchange={(event) => saveBlur(Number(event.currentTarget.value))}
      />
      <output for="glass-blur">{blur} %</output>
    </span>
  </div>
  <p class="hint">
    {#if appearance.theme === "glass"}
      100 % : tuiles en verre dépoli. 0 % : tuiles transparentes, sans flou.
    {:else}
      Le flou ne concerne que le thème Verre.
    {/if}
  </p>
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

  .themes {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    padding: 0.5rem 0;
  }

  .theme {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.5rem;
    border: 2px solid transparent;
    border-radius: 0.75rem;
    font-size: 0.875rem;
    text-align: center;
    cursor: pointer;
  }

  .theme.selected {
    border-color: var(--color-accent);
  }

  .theme:has(input:focus-visible) {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  input[type="radio"] {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  /* Petit aperçu de chaque thème : trois tuiles sur son fond. */
  .preview {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 4px;
    width: 8rem;
    height: 4.5rem;
    padding: 6px;
    border-radius: 0.5rem;
  }

  .preview span {
    border-radius: 4px;
  }

  .preview span:first-child {
    grid-row: span 2;
  }

  .preview.classic {
    background: #121418;
  }

  .preview.classic span {
    border: 1px solid #2e333c;
    border-top: 2px solid #4f8cff;
    background: #1c1f25;
  }

  .preview.glass {
    background:
      radial-gradient(70% 80% at 0% 0%, #e0568a, transparent 70%),
      radial-gradient(70% 80% at 100% 100%, #10a88c, transparent 70%), #4f6cf0;
  }

  .preview.glass span {
    border: 1px solid rgb(255 255 255 / 35%);
    border-radius: 6px;
    background: rgb(255 255 255 / 22%);
  }

  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.5rem 0;
  }

  select {
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: var(--color-surface-hover);
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  .slider {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  input[type="range"] {
    width: 12rem;
    accent-color: var(--color-accent);
    cursor: pointer;
  }

  input[type="range"]:disabled {
    opacity: 0.5;
    cursor: default;
  }

  output {
    min-width: 3.5rem;
    color: var(--color-text);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  select:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }
</style>
