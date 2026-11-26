<!--
  Une ligne de mesure : nom, barre de remplissage et valeur.
  La barre passe à l'orange au-delà de 85 % et au rouge au-delà de 95 %.
-->
<script lang="ts">
  let { label, value, detail }: { label: string; value: number; detail: string } = $props();

  let level = $derived(value >= 95 ? "danger" : value >= 85 ? "warning" : "normal");
</script>

<div class="row" data-level={level}>
  <span class="label">{label}</span>
  <span
    class="bar"
    role="meter"
    aria-label={label}
    aria-valuenow={Math.round(value)}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-valuetext={detail}
  >
    <span class="fill" style:width="{value}%"></span>
  </span>
  <span class="detail">{detail}</span>
</div>

<style>
  .row {
    display: contents;
    --bar-color: var(--tile-color);
  }

  .row[data-level="warning"] {
    --bar-color: var(--color-warning);
  }

  .row[data-level="danger"] {
    --bar-color: var(--color-danger);
  }

  .label,
  .detail {
    white-space: nowrap;
  }

  .detail {
    color: var(--color-text);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .bar {
    height: 0.45rem;
    overflow: hidden;
    border-radius: 999px;
    background: var(--color-bg);
  }

  .fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--bar-color);
    transition: width 0.4s ease;
  }
</style>
