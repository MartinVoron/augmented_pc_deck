// Préparation commune des tests de l'interface (voir vitest.config.ts).

// jsdom ne connaît pas `matchMedia`, dont se sert `svelte/motion` (mouvements réduits).
if (typeof window.matchMedia !== "function") {
  window.matchMedia = (query: string): MediaQueryList =>
    ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      dispatchEvent: () => false,
    }) as MediaQueryList;
}
