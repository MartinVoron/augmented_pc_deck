// Tests du calcul de version. Lancer avec : node --test scripts/version/version.test.mjs
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync, mkdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  ajouterAuJournal,
  analyserCommit,
  etat,
  extraireNotes,
  monteeDesCommits,
  monterVersion,
  preparer,
  redigerSection,
  remplacerVersionCargo,
  remplacerVersionJson,
  remplacerVersionVerrouCargo,
  verifierVersions,
} from "./version.mjs";

test("lit un message de commit en français", () => {
  const c = analyserCommit("ajout(minuteur): ajouter un minuteur pomodoro (#12)", "abc");
  assert.deepEqual(c, {
    type: "ajout",
    zone: "minuteur",
    description: "ajouter un minuteur pomodoro",
    pr: "12",
    cassant: false,
    noteCassante: null,
    sha: "abc",
  });
  assert.equal(analyserCommit("feat: add things"), null);
  assert.equal(analyserCommit("Merge pull request #3 from x"), null);
});

test("repère les changements cassants (! ou note dans le corps)", () => {
  assert.equal(analyserCommit("ajout(reglages)!: changer le format").cassant, true);
  const c = analyserCommit(
    "refonte: changer la config\n\nCHANGEMENT CASSANT : l'ancien fichier doit être migré.",
  );
  assert.equal(c.cassant, true);
  assert.equal(c.noteCassante, "l'ancien fichier doit être migré.");
});

test("choisit la montée de version la plus forte", () => {
  const commits = (...messages) => messages.map((m) => analyserCommit(m));
  assert.equal(monteeDesCommits(commits("doc: expliquer", "ci: lancer")), "aucune");
  assert.equal(monteeDesCommits(commits("doc: a", "correction: b")), "corrective");
  assert.equal(monteeDesCommits(commits("perf: a", "annulation: b")), "corrective");
  assert.equal(monteeDesCommits(commits("correction: a", "ajout: b")), "mineure");
  assert.equal(monteeDesCommits(commits("ajout: a", "refonte!: b")), "majeure");
});

test("applique SemVer, avec une règle douce avant la 1.0.0", () => {
  assert.equal(monterVersion("0.1.0", "corrective"), "0.1.1");
  assert.equal(monterVersion("0.1.3", "mineure"), "0.2.0");
  assert.equal(monterVersion("0.4.2", "majeure"), "0.5.0");
  assert.equal(monterVersion("1.2.3", "majeure"), "2.0.0");
  assert.equal(monterVersion("1.2.3", "aucune"), "1.2.3");
  assert.throws(() => monterVersion("1.2", "mineure"));
});

test("rédige une section de journal en français, rangée par rubrique", () => {
  const commits = [
    analyserCommit("correction(lanceur): corriger le chemin (#7)", "1111111aaaa"),
    analyserCommit("ajout(minuteur): ajouter un minuteur (#5)", "2222222bbbb"),
    analyserCommit("doc: expliquer l'installation", "3333333cccc"),
    analyserCommit("ajout: ajouter les tuiles", "4444444dddd"),
  ];
  const section = redigerSection({
    version: "0.2.0",
    precedente: "0.1.0",
    date: "2026-11-26",
    commits,
    depot: "https://github.com/moi/depot",
  });
  assert.equal(
    section,
    [
      "## [0.2.0](https://github.com/moi/depot/compare/v0.1.0...v0.2.0) - 2026-11-26",
      "",
      "### Nouveautés",
      "",
      "- **minuteur :** ajouter un minuteur ([#5](https://github.com/moi/depot/pull/5))",
      "- ajouter les tuiles ([4444444](https://github.com/moi/depot/commit/4444444dddd))",
      "",
      "### Corrections",
      "",
      "- **lanceur :** corriger le chemin ([#7](https://github.com/moi/depot/pull/7))",
      "",
    ].join("\n"),
  );
});

test("ajoute les sections en tête du journal et les relit", () => {
  let journal = ajouterAuJournal("", "## [0.1.0] - 2026-01-01\n\n### Nouveautés\n\n- a\n");
  journal = ajouterAuJournal(journal, "## [0.2.0](lien) - 2026-02-01\n\n### Corrections\n\n- b\n");
  assert.ok(journal.startsWith("# Journal des modifications"));
  assert.ok(journal.indexOf("## [0.2.0]") < journal.indexOf("## [0.1.0]"));
  assert.equal(extraireNotes(journal, "0.2.0"), "### Corrections\n\n- b");
  assert.equal(extraireNotes(journal, "0.1.0"), "### Nouveautés\n\n- a");
  assert.equal(extraireNotes(journal, "0.1.1"), null);
  assert.equal(extraireNotes(journal, "0.1"), null);
});

test("change la version sans toucher à la mise en forme", () => {
  const conf = '{\n  "productName": "X",\n  "version": "0.1.0",\n  "targets": ["nsis", "msi"]\n}\n';
  assert.equal(
    remplacerVersionJson(conf, "0.2.0"),
    '{\n  "productName": "X",\n  "version": "0.2.0",\n  "targets": ["nsis", "msi"]\n}\n',
  );
  const cargo =
    '[package]\nname = "app"\nversion = "0.1.0"\n\n[dependencies]\nserde = { version = "1" }\n';
  assert.equal(
    remplacerVersionCargo(cargo, "0.2.0"),
    '[package]\nname = "app"\nversion = "0.2.0"\n\n[dependencies]\nserde = { version = "1" }\n',
  );
  const verrou =
    '[[package]]\nname = "apparent"\nversion = "9.9.9"\n\n[[package]]\nname = "app"\nversion = "0.1.0"\n';
  assert.equal(
    remplacerVersionVerrouCargo(verrou, "app", "0.2.0"),
    '[[package]]\nname = "apparent"\nversion = "9.9.9"\n\n[[package]]\nname = "app"\nversion = "0.2.0"\n',
  );
});

// ---------------------------------------------------------------------------
// Scénario complet dans un vrai dépôt Git temporaire

function creerDepot() {
  const racine = mkdtempSync(join(tmpdir(), "version-"));
  const git = (...args) => execFileSync("git", args, { cwd: racine, encoding: "utf8" });
  git("init", "-q", "-b", "main");
  git("config", "user.name", "Test");
  git("config", "user.email", "test@example.com");
  git("config", "commit.gpgsign", "false");
  git("config", "core.hooksPath", "/dev/null");
  mkdirSync(join(racine, "src-tauri"));
  const fichiers = {
    "package.json": '{\n  "name": "app",\n  "version": "0.1.0"\n}\n',
    "package-lock.json":
      '{\n  "name": "app",\n  "version": "0.1.0",\n  "packages": {\n    "": {\n      "version": "0.1.0"\n    }\n  }\n}\n',
    "src-tauri/Cargo.toml": '[package]\nname = "app"\nversion = "0.1.0"\n',
    "src-tauri/Cargo.lock": 'version = 4\n\n[[package]]\nname = "app"\nversion = "0.1.0"\n',
    "src-tauri/tauri.conf.json": '{\n  "productName": "App",\n  "version": "0.1.0"\n}\n',
  };
  for (const [chemin, contenu] of Object.entries(fichiers))
    writeFileSync(join(racine, chemin), contenu);
  const commit = (message) => {
    git("add", "-A");
    git("commit", "-q", "--allow-empty", "-m", message);
  };
  return { racine, git, commit };
}

test("prépare la première version, la publie, puis calcule la suivante", (t) => {
  const { racine, git, commit } = creerDepot();
  t.after(() => rmSync(racine, { recursive: true, force: true }));

  commit("ajout(tauri): initialiser l'application (#33)");
  assert.equal(etat(racine).action, "preparer");

  // Première version : on garde 0.1.0, déjà écrite dans les fichiers.
  const r1 = preparer(racine);
  assert.equal(r1.version, "0.1.0");
  assert.match(readFileSync(join(racine, "CHANGELOG.md"), "utf8"), /## \[0\.1\.0\]/);
  commit("maintenance: publier la version 0.1.0 (#40)");

  // Après fusion de la PR de version : il faut publier.
  const r2 = etat(racine);
  assert.equal(r2.action, "publier");
  assert.equal(r2.version, "0.1.0");
  assert.match(r2.notes, /initialiser l'application/);
  git("tag", "v0.1.0");
  assert.equal(etat(racine).action, "rien");

  // Des commits sans effet visible ne déclenchent rien.
  commit("doc: expliquer l'installation");
  assert.equal(etat(racine).action, "rien");

  // Une correction donne 0.1.1, un ajout 0.2.0.
  commit("correction(lanceur): corriger le chemin");
  assert.equal(etat(racine).version, "0.1.1");
  commit("ajout(minuteur): ajouter un minuteur");
  const r3 = preparer(racine);
  assert.equal(r3.version, "0.2.0");
  assert.doesNotMatch(r3.section, /expliquer l'installation/);
  assert.deepEqual(
    verifierVersions(racine).versions.map((v) => v.version),
    ["0.2.0", "0.2.0", "0.2.0", "0.2.0", "0.2.0"],
  );
  const journal = readFileSync(join(racine, "CHANGELOG.md"), "utf8");
  assert.ok(journal.indexOf("## [0.2.0]") < journal.indexOf("## [0.1.0]"));

  // On peut forcer la 1.0.0, mais pas revenir en arrière.
  assert.equal(etat(racine, { forcer: "1.0.0" }).version, "1.0.0");
  assert.throws(() => etat(racine, { forcer: "0.0.9" }), /plus grande/);
});

test("ne fait rien tant que l'application n'existe pas", (t) => {
  const racine = mkdtempSync(join(tmpdir(), "version-vide-"));
  t.after(() => rmSync(racine, { recursive: true, force: true }));
  assert.equal(etat(racine).action, "rien");
  assert.deepEqual(verifierVersions(racine), { versions: [], ok: true });
});

test("signale des versions différentes", (t) => {
  const { racine } = creerDepot();
  t.after(() => rmSync(racine, { recursive: true, force: true }));
  writeFileSync(
    join(racine, "src-tauri/Cargo.toml"),
    '[package]\nname = "app"\nversion = "0.3.0"\n',
  );
  assert.equal(verifierVersions(racine).ok, false);
});
