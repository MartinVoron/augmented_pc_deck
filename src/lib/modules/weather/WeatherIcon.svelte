<!-- Icône d'un temps (soleil, nuages, pluie…), de jour ou de nuit. -->
<script lang="ts">
  import {
    Cloud,
    CloudDrizzle,
    CloudFog,
    CloudLightning,
    CloudMoon,
    CloudRain,
    CloudSnow,
    CloudSun,
    Moon,
    Sun,
  } from "@lucide/svelte";
  import { describe } from ".";

  let {
    code,
    isDay = true,
    size = 24,
  }: { code: number; isDay?: boolean; size?: number } = $props();

  const ICONS = {
    clear: [Sun, Moon],
    partly: [CloudSun, CloudMoon],
    cloudy: [Cloud, Cloud],
    fog: [CloudFog, CloudFog],
    drizzle: [CloudDrizzle, CloudDrizzle],
    rain: [CloudRain, CloudRain],
    snow: [CloudSnow, CloudSnow],
    storm: [CloudLightning, CloudLightning],
  };

  let Icon = $derived(ICONS[describe(code).icon][isDay ? 0 : 1]);
</script>

<span class="weather-icon" data-icon={describe(code).icon} aria-hidden="true">
  <Icon {size} />
</span>

<style>
  .weather-icon {
    display: inline-flex;
    color: var(--tile-color);
  }

  [data-icon="clear"],
  [data-icon="partly"] {
    color: var(--color-warning);
  }
</style>
