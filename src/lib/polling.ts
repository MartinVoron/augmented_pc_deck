/**
 * Appelle `task` tout de suite, puis toutes les `intervalMs` millisecondes.
 * Les appels sont suspendus quand la fenêtre est masquée (zone de notification),
 * pour ne pas charger le PC pour rien, et reprennent dès qu'elle réapparaît.
 * Deux appels ne se chevauchent jamais. Renvoie une fonction qui arrête tout.
 */
export function poll(task: () => Promise<void> | void, intervalMs: number): () => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let stopped = false;

  async function tick() {
    timer = undefined;
    if (stopped || document.hidden) return;
    try {
      await task();
    } finally {
      if (!stopped && !document.hidden) timer = setTimeout(tick, intervalMs);
    }
  }

  function onVisibilityChange() {
    if (!document.hidden && timer === undefined) void tick();
  }

  document.addEventListener("visibilitychange", onVisibilityChange);
  void tick();

  return () => {
    stopped = true;
    clearTimeout(timer);
    document.removeEventListener("visibilitychange", onVisibilityChange);
  };
}
