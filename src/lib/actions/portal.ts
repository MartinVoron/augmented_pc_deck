// Déplace un élément à la fin de <body>, hors de la tuile qui l'affiche.

/**
 * Action Svelte : place `node` directement dans `<body>`. Une fenêtre de dialogue en
 * `position: fixed` reste sinon prisonnière d'une tuile du thème Verre, car le
 * `backdrop-filter` de la tuile devient son cadre de référence.
 */
export function portal(node: HTMLElement): { destroy(): void } {
  document.body.appendChild(node);
  return {
    destroy() {
      node.remove();
    },
  };
}
