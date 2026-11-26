<!--
  Icône d'un raccourci : l'icône extraite de la cible par Windows,
  ou à défaut une icône générique (globe pour une URL, fenêtre sinon).
-->
<script lang="ts">
  import { AppWindow, Globe } from "@lucide/svelte";
  import { getIcon, isUrl } from ".";

  let { target, size = 32 }: { target: string; size?: number } = $props();

  let src = $state<string | null>(null);

  $effect(() => {
    let current = true;
    src = null;
    void getIcon(target).then((icon) => {
      if (current) src = icon;
    });
    return () => (current = false);
  });
</script>

{#if src}
  <img {src} alt="" width={size} height={size} onerror={() => (src = null)} />
{:else if isUrl(target)}
  <Globe {size} />
{:else}
  <AppWindow {size} />
{/if}

<style>
  img {
    object-fit: contain;
  }
</style>
