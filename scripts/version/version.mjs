#!/usr/bin/env node
// Calcul automatique des versions (SemVer) à partir des messages de commit en français.
// Voir CONTRIBUTING.md, section « Versions ».
//
// Commandes :
//   node scripts/version/version.mjs etat [--forcer X.Y.Z]
//       Dit ce qu'il faut faire : « publier » (la version du dépôt n'a pas encore
//       d'étiquette), « preparer » (des commits attendent une nouvelle version) ou « rien ».
//   node scripts/version/version.mjs preparer [--forcer X.Y.Z]
//       Écrit la nouvelle version dans tous les fichiers et complète CHANGELOG.md.
//   node scripts/version/version.mjs notes X.Y.Z
//       Affiche les notes de la version X.Y.Z, tirées de CHANGELOG.md.
//   node scripts/version/version.mjs verifier
//       Vérifie que tous les fichiers portent le même numéro de version.
//
// Aucune dépendance : seulement Node.js (20 ou plus) et Git.

import { execFileSync } from "node:child_process";
import { appendFileSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// ---------------------------------------------------------------------------
// Règles de version

/** Rang des montées de version : plus le nombre est grand, plus la montée est forte. */
const RANG = { aucune: 0, corrective: 1, mineure: 2, majeure: 3 };

/** Montée de version provoquée par chaque type de commit. Les autres types n'en provoquent pas. */
const MONTEE_PAR_TYPE = {
  ajout: "mineure",
  correction: "corrective",
  perf: "corrective",
  annulation: "corrective",
};

/** Rubriques du journal des modifications, dans l'ordre d'affichage. */
const RUBRIQUES = [
  { type: "ajout", titre: "Nouveautés" },
  { type: "correction", titre: "Corrections" },
  { type: "perf", titre: "Performances" },
  { type: "annulation", titre: "Annulations" },
];

const TYPES = "ajout|correction|doc|style|refonte|perf|test|build|ci|maintenance|annulation";
const ENTETE = new RegExp(`^(${TYPES})(?:\\(([a-z0-9][a-z0-9-]*)\\))?(!)?: (.+)$`);
const NOTE_CASSANTE = /^(?:CHANGEMENT CASSANT|BREAKING[ -]CHANGE)\s*:\s*(.+)$/m;
const NUMERO_PR = /\s*\(#(\d+)\)$/;

const ENTETE_JOURNAL = `# Journal des modifications

Toutes les évolutions notables d'Augmented PC Deck sont listées ici, de la plus récente
à la plus ancienne. Ce fichier est écrit automatiquement à partir des messages de commit
(voir \`CONTRIBUTING.md\`) : ne pas le modifier à la main, sauf pour corriger une coquille.
`;

/**
 * Lit un message de commit. Seule la première ligne compte pour le type ; le corps
 * sert uniquement à repérer un changement cassant.
 * @returns {{type: string, zone: string|null, description: string, pr: string|null,
 *   cassant: boolean, noteCassante: string|null, sha: string}|null}
 */
export function analyserCommit(message, sha = "") {
  const [entete = "", ...reste] = message.trim().split(/\r?\n/);
  const trouve = ENTETE.exec(entete.trim());
  if (!trouve) return null;
  const [, type, zone = null, point, texte] = trouve;
  const pr = NUMERO_PR.exec(texte);
  const corps = reste.join("\n");
  const note = NOTE_CASSANTE.exec(corps);
  return {
    type,
    zone,
    description: texte.replace(NUMERO_PR, ""),
    pr: pr ? pr[1] : null,
    cassant: Boolean(point) || Boolean(note),
    noteCassante: note ? note[1].trim() : null,
    sha,
  };
}

/** Montée de version d'un commit déjà analysé. */
export function monteeDuCommit(commit) {
  if (commit.cassant) return "majeure";
  return MONTEE_PAR_TYPE[commit.type] ?? "aucune";
}

/** Montée de version la plus forte parmi une liste de commits. */
export function monteeDesCommits(commits) {
  let montee = "aucune";
  for (const commit of commits) {
    const m = monteeDuCommit(commit);
    if (RANG[m] > RANG[montee]) montee = m;
  }
  return montee;
}

export function lireVersion(texte) {
  const trouve = /^(\d+)\.(\d+)\.(\d+)$/.exec(String(texte).trim());
  if (!trouve) throw new Error(`Numéro de version invalide : « ${texte} » (attendu : X.Y.Z)`);
  return trouve.slice(1).map(Number);
}

/**
 * Applique une montée de version. Avant la 1.0.0, un changement cassant ne fait monter
 * que la version mineure (pratique courante en SemVer pour les versions 0.x).
 */
export function monterVersion(version, montee) {
  const [majeur, mineur, correctif] = lireVersion(version);
  const effective = montee === "majeure" && majeur === 0 ? "mineure" : montee;
  switch (effective) {
    case "majeure":
      return `${majeur + 1}.0.0`;
    case "mineure":
      return `${majeur}.${mineur + 1}.0`;
    case "corrective":
      return `${majeur}.${mineur}.${correctif + 1}`;
    default:
      return version;
  }
}

export function comparerVersions(a, b) {
  const va = lireVersion(a);
  const vb = lireVersion(b);
  for (let i = 0; i < 3; i++) if (va[i] !== vb[i]) return va[i] - vb[i];
  return 0;
}

// ---------------------------------------------------------------------------
// Journal des modifications

function ligneDuJournal(commit, depot) {
  const zone = commit.zone ? `**${commit.zone} :** ` : "";
  let lien = "";
  if (commit.pr)
    lien = depot ? ` ([#${commit.pr}](${depot}/pull/${commit.pr}))` : ` (#${commit.pr})`;
  else if (commit.sha) {
    const court = commit.sha.slice(0, 7);
    lien = depot ? ` ([${court}](${depot}/commit/${commit.sha}))` : ` (${court})`;
  }
  return `- ${zone}${commit.description}${lien}`;
}

/**
 * Rédige la section d'une version pour CHANGELOG.md.
 * @param {{version: string, precedente: string|null, date: string, commits: object[],
 *   depot?: string|null}} options
 */
export function redigerSection({ version, precedente, date, commits, depot = null }) {
  let lienVersion = "";
  if (depot) {
    lienVersion = precedente
      ? `(${depot}/compare/v${precedente}...v${version})`
      : `(${depot}/releases/tag/v${version})`;
  }
  const lignes = [`## [${version}]${lienVersion} - ${date}`, ""];

  const cassants = commits.filter((c) => c.cassant);
  if (cassants.length > 0) {
    lignes.push("### ⚠ Changements cassants", "");
    for (const c of cassants) {
      const texte = c.noteCassante ? { ...c, description: c.noteCassante } : c;
      lignes.push(ligneDuJournal(texte, depot));
    }
    lignes.push("");
  }

  for (const { type, titre } of RUBRIQUES) {
    const liste = commits.filter((c) => c.type === type);
    if (liste.length === 0) continue;
    lignes.push(`### ${titre}`, "");
    for (const c of liste) lignes.push(ligneDuJournal(c, depot));
    lignes.push("");
  }

  if (lignes.length === 2) lignes.push("Aucun changement visible : version de maintenance.", "");
  return lignes.join("\n");
}

/** Ajoute une section en tête du journal (sous l'en-tête), en créant le journal si besoin. */
export function ajouterAuJournal(journal, section) {
  const base = journal && journal.trim() ? journal : ENTETE_JOURNAL;
  const debut = base.search(/^## /m);
  if (debut === -1) return `${base.trimEnd()}\n\n${section.trimEnd()}\n`;
  return `${base.slice(0, debut)}${section.trimEnd()}\n\n${base.slice(debut)}`;
}

/** Retrouve les notes d'une version dans le journal (sans la ligne de titre), ou null. */
export function extraireNotes(journal, version) {
  const echappee = version.replace(/\./g, "\\.");
  const titre = new RegExp(`^## \\[?${echappee}\\]?(?:\\(|[ \\t]|$).*$`, "m");
  const trouve = titre.exec(journal ?? "");
  if (!trouve) return null;
  const suite = journal.slice(trouve.index + trouve[0].length);
  const fin = suite.search(/^## /m);
  return (fin === -1 ? suite : suite.slice(0, fin)).trim();
}

// ---------------------------------------------------------------------------
// Fichiers qui portent le numéro de version

/** Remplace la première clé "version" d'un JSON sans toucher au reste de la mise en forme. */
export function remplacerVersionJson(texte, version) {
  const nouveau = texte.replace(/("version"\s*:\s*")[^"]*(")/, `$1${version}$2`);
  if (JSON.parse(nouveau).version !== version) {
    throw new Error("la clé « version » de premier niveau n'a pas été trouvée");
  }
  return nouveau;
}

/** package-lock.json : la version apparaît à la racine et dans packages[""]. */
export function remplacerVersionVerrouNpm(texte, version) {
  const donnees = JSON.parse(texte);
  donnees.version = version;
  if (donnees.packages?.[""]) donnees.packages[""].version = version;
  return `${JSON.stringify(donnees, null, 2)}\n`;
}

/** Cargo.toml : la clé version de la section [package]. */
export function remplacerVersionCargo(texte, version) {
  const motif = /(^\[package\][^[]*?^version\s*=\s*")[^"]*(")/m;
  if (!motif.test(texte)) throw new Error("version introuvable dans la section [package]");
  return texte.replace(motif, `$1${version}$2`);
}

export function nomDuCrate(cargoToml) {
  const trouve = /^\[package\][^[]*?^name\s*=\s*"([^"]+)"/m.exec(cargoToml);
  if (!trouve) throw new Error("nom introuvable dans la section [package] de Cargo.toml");
  return trouve[1];
}

/** Cargo.lock : la version du paquet de l'application. */
export function remplacerVersionVerrouCargo(texte, nom, version) {
  const echappe = nom.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const motif = new RegExp(
    `(\\[\\[package\\]\\]\\r?\\nname = "${echappe}"\\r?\\nversion = ")[^"]*(")`,
  );
  if (!motif.test(texte)) throw new Error(`paquet « ${nom} » introuvable`);
  return texte.replace(motif, `$1${version}$2`);
}

function versionCargo(texte) {
  return /^\[package\][^[]*?^version\s*=\s*"([^"]*)"/m.exec(texte)?.[1] ?? null;
}

function versionVerrouCargo(texte, nom) {
  const echappe = nom.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const motif = new RegExp(
    `\\[\\[package\\]\\]\\r?\\nname = "${echappe}"\\r?\\nversion = "([^"]*)"`,
  );
  return motif.exec(texte)?.[1] ?? null;
}

/**
 * Liste des fichiers versionnés, avec comment lire et écrire leur version.
 * tauri.conf.json peut aussi pointer vers package.json ("version": "../package.json") :
 * dans ce cas il suit tout seul et on n'y touche pas.
 */
function fichiersVersionnes(racine) {
  const chemin = (p) => join(racine, p);
  const cargoToml = chemin("src-tauri/Cargo.toml");
  const nom = () => nomDuCrate(readFileSync(cargoToml, "utf8"));
  return [
    {
      fichier: "package.json",
      lire: (t) => JSON.parse(t).version,
      ecrire: remplacerVersionJson,
    },
    {
      fichier: "package-lock.json",
      lire: (t) => {
        const d = JSON.parse(t);
        return d.packages?.[""] && d.packages[""].version !== d.version
          ? "(incohérent)"
          : d.version;
      },
      ecrire: remplacerVersionVerrouNpm,
    },
    { fichier: "src-tauri/Cargo.toml", lire: versionCargo, ecrire: remplacerVersionCargo },
    {
      fichier: "src-tauri/Cargo.lock",
      lire: (t) => versionVerrouCargo(t, nom()),
      ecrire: (t, v) => remplacerVersionVerrouCargo(t, nom(), v),
    },
    {
      fichier: "src-tauri/tauri.conf.json",
      lire: (t) => JSON.parse(t).version,
      ecrire: remplacerVersionJson,
      suitPackageJson: (t) => /\.json$/.test(JSON.parse(t).version ?? ""),
    },
  ].map((f) => ({ ...f, chemin: chemin(f.fichier) }));
}

/** Écrit la version dans tous les fichiers présents. Renvoie la liste des fichiers modifiés. */
export function ecrireVersion(racine, version) {
  const modifies = [];
  for (const f of fichiersVersionnes(racine)) {
    if (!existsSync(f.chemin)) continue;
    const texte = readFileSync(f.chemin, "utf8");
    if (f.suitPackageJson?.(texte)) continue;
    try {
      writeFileSync(f.chemin, f.ecrire(texte, version));
    } catch (erreur) {
      throw new Error(`${f.fichier} : ${erreur.message}`);
    }
    modifies.push(f.fichier);
  }
  return modifies;
}

/** Compare les versions de tous les fichiers présents. */
export function verifierVersions(racine) {
  const versions = [];
  for (const f of fichiersVersionnes(racine)) {
    if (!existsSync(f.chemin)) continue;
    const texte = readFileSync(f.chemin, "utf8");
    if (f.suitPackageJson?.(texte)) continue;
    versions.push({ fichier: f.fichier, version: f.lire(texte) });
  }
  const reference = versions[0]?.version;
  const ecarts = versions.filter((v) => v.version !== reference);
  return { versions, ok: ecarts.length === 0 };
}

// ---------------------------------------------------------------------------
// Git

function git(racine, ...args) {
  return execFileSync("git", args, {
    cwd: racine,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  }).trim();
}

/** Dernière étiquette de version vX.Y.Z atteignable depuis HEAD, ou null. */
export function derniereEtiquette(racine) {
  try {
    return git(racine, "describe", "--tags", "--abbrev=0", "--match", "v[0-9]*.[0-9]*.[0-9]*");
  } catch {
    return null;
  }
}

function etiquetteExiste(racine, etiquette) {
  try {
    git(racine, "rev-parse", "--verify", "--quiet", `refs/tags/${etiquette}`);
    return true;
  } catch {
    return false;
  }
}

/** Commits depuis l'étiquette donnée (ou depuis le début), les plus anciens en premier. */
export function commitsDepuis(racine, etiquette) {
  const plage = etiquette ? [`${etiquette}..HEAD`] : ["HEAD"];
  const sortie = execFileSync(
    "git",
    ["log", "--no-merges", "--reverse", "--format=%H%x1f%B%x1e", ...plage],
    { cwd: racine, encoding: "utf8" },
  );
  return sortie
    .split("\x1e")
    .map((bloc) => bloc.trim())
    .filter(Boolean)
    .map((bloc) => {
      const [sha, message = ""] = bloc.split("\x1f");
      return analyserCommit(message, sha.trim());
    })
    .filter(Boolean);
}

function adresseDuDepot() {
  const { GITHUB_SERVER_URL: serveur, GITHUB_REPOSITORY: depot } = process.env;
  return serveur && depot ? `${serveur}/${depot}` : null;
}

// ---------------------------------------------------------------------------
// Commandes

/**
 * Décide de la prochaine étape.
 * - publier : la version des fichiers a une section dans CHANGELOG.md mais pas encore
 *   d'étiquette (cas juste après la fusion d'une PR de version) ;
 * - preparer : des commits visibles attendent une nouvelle version (ou une version est forcée) ;
 * - rien : sinon.
 */
export function etat(racine, { forcer = null } = {}) {
  const paquet = join(racine, "package.json");
  if (!existsSync(paquet)) {
    return { action: "rien", raison: "package.json absent : l'application n'est pas encore créée" };
  }
  const actuelle = JSON.parse(readFileSync(paquet, "utf8")).version;
  const cheminJournal = join(racine, "CHANGELOG.md");
  const journal = existsSync(cheminJournal) ? readFileSync(cheminJournal, "utf8") : "";

  if (!forcer && !etiquetteExiste(racine, `v${actuelle}`)) {
    const notes = extraireNotes(journal, actuelle);
    if (notes !== null) return { action: "publier", version: actuelle, notes };
  }

  const etiquette = derniereEtiquette(racine);
  const precedente = etiquette ? etiquette.replace(/^v/, "") : null;
  const commits = commitsDepuis(racine, etiquette);
  const montee = monteeDesCommits(commits);

  let version;
  if (forcer) {
    lireVersion(forcer);
    if (precedente && comparerVersions(forcer, precedente) <= 0) {
      throw new Error(`La version forcée ${forcer} doit être plus grande que ${precedente}`);
    }
    version = forcer;
  } else if (montee === "aucune") {
    return { action: "rien", raison: "aucun commit ajout, correction, perf ou annulation" };
  } else {
    // Première version : on garde celle déjà écrite dans les fichiers (0.1.0 au départ).
    version = precedente ? monterVersion(precedente, montee) : actuelle;
  }

  const section = redigerSection({
    version,
    precedente,
    date: new Date().toISOString().slice(0, 10),
    commits,
    depot: adresseDuDepot(),
  });
  return { action: "preparer", version, precedente, montee, section, commits };
}

export function preparer(racine, options) {
  const resultat = etat(racine, options);
  if (resultat.action !== "preparer") return resultat;
  const modifies = ecrireVersion(racine, resultat.version);
  const cheminJournal = join(racine, "CHANGELOG.md");
  const journal = existsSync(cheminJournal) ? readFileSync(cheminJournal, "utf8") : "";
  writeFileSync(cheminJournal, ajouterAuJournal(journal, resultat.section));
  return { ...resultat, modifies: [...modifies, "CHANGELOG.md"] };
}

/** Écrit les sorties pour GitHub Actions (variable GITHUB_OUTPUT), si elle existe. */
function sortieActions(valeurs) {
  const fichier = process.env.GITHUB_OUTPUT;
  if (!fichier) return;
  for (const [cle, valeur] of Object.entries(valeurs)) {
    if (valeur === undefined || valeur === null) continue;
    const texte = String(valeur);
    if (texte.includes("\n")) {
      const fin = `FIN_${Math.random().toString(36).slice(2)}`;
      appendFileSync(fichier, `${cle}<<${fin}\n${texte}\n${fin}\n`);
    } else {
      appendFileSync(fichier, `${cle}=${texte}\n`);
    }
  }
}

function principal(args) {
  const racine = process.cwd();
  const [commande, ...reste] = args;
  const indice = reste.indexOf("--forcer");
  const forcer = indice === -1 ? null : reste[indice + 1] || null;

  switch (commande) {
    case "etat":
    case "preparer": {
      const r = commande === "etat" ? etat(racine, { forcer }) : preparer(racine, { forcer });
      if (r.action === "rien") console.log(`Rien à faire : ${r.raison}.`);
      else if (r.action === "publier") console.log(`Version ${r.version} prête à être publiée.`);
      else {
        const depuis = r.precedente ? `après ${r.precedente}` : "première version";
        console.log(`Prochaine version : ${r.version} (${depuis}, montée ${r.montee}).`);
        if (r.modifies) console.log(`Fichiers mis à jour : ${r.modifies.join(", ")}`);
        console.log(`\n${r.section}`);
      }
      sortieActions({ action: r.action, version: r.version, notes: r.notes ?? r.section });
      return 0;
    }
    case "notes": {
      const version = reste[0];
      const journal = existsSync("CHANGELOG.md") ? readFileSync("CHANGELOG.md", "utf8") : "";
      const notes = version ? extraireNotes(journal, version) : null;
      if (notes === null) {
        console.error(`✖ Aucune section pour la version « ${version ?? ""} » dans CHANGELOG.md`);
        return 1;
      }
      console.log(notes);
      return 0;
    }
    case "verifier": {
      const { versions, ok } = verifierVersions(racine);
      if (versions.length === 0) {
        console.log("Aucun fichier versionné pour l'instant.");
        return 0;
      }
      for (const v of versions) console.log(`${ok ? "✔" : "•"} ${v.fichier} : ${v.version}`);
      if (!ok) {
        console.error(
          "✖ Les numéros de version ne sont pas les mêmes partout. " +
            "Ne les modifiez pas à la main : ils sont gérés par la PR de version.",
        );
        return 1;
      }
      return 0;
    }
    default:
      console.error("Usage : node scripts/version/version.mjs <etat|preparer|notes|verifier>");
      return 2;
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    process.exitCode = principal(process.argv.slice(2));
  } catch (erreur) {
    console.error(`✖ ${erreur.message}`);
    process.exitCode = 1;
  }
}
