// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Fails on a Tailwind class in the source that no longer makes any CSS: a
// token renamed or dropped from `src/styles` leaves its old classes behind,
// and Tailwind skips them silently, so a window loses its spacing or colour
// with nothing to say so. A class is checked where it sits among others in a
// string that is mostly working classes (a `className`, a `cn(...)` argument,
// a variant), and only when it starts like a utility (`gap-`, `text-`, ...)
// and is not a class of the app's own stylesheets, so prose, ids and marker
// classes such as `group` are left alone.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import process from "node:process";

import { __unstable__loadDesignSystem } from "tailwindcss";

const root = process.cwd();
const entry = join(root, "src/index.css");

function files(directory, pattern) {
  return readdirSync(directory, { recursive: true, withFileTypes: true })
    .filter((item) => item.isFile() && pattern.test(item.name))
    .map((item) => join(item.parentPath, item.name))
    .filter((file) => !file.includes(`${join("src", "bindings")}`));
}

// Resolves `@import`s the way the Vite plugin does: relative paths from the
// importing file, packages through their `style` entry. The package is found
// on disk, since its `exports` need not list `package.json`.
function packageDirectory(name, base) {
  for (let directory = base; ; directory = dirname(directory)) {
    const candidate = join(directory, "node_modules", name);
    if (existsSync(join(candidate, "package.json"))) return candidate;
    if (dirname(directory) === directory)
      throw new Error(`Cannot find ${name} from ${base}`);
  }
}

async function loadStylesheet(id, base) {
  let path;
  if (id.startsWith(".") || id.startsWith("/")) {
    path = resolve(base, id);
  } else {
    const parts = id.split("/");
    const name = id.startsWith("@") ? parts.slice(0, 2).join("/") : parts[0];
    const subpath = id.slice(name.length + 1);
    const directory = packageDirectory(name, base);
    const manifest = JSON.parse(
      readFileSync(join(directory, "package.json"), "utf8"),
    );
    const main =
      manifest.exports?.["."]?.style ?? manifest.style ?? manifest.main;
    path = join(directory, subpath || main);
  }
  return { base: dirname(path), content: readFileSync(path, "utf8"), path };
}

const design = await __unstable__loadDesignSystem(readFileSync(entry, "utf8"), {
  base: dirname(entry),
  loadStylesheet,
});

const roots = [
  ...design.utilities.keys("static"),
  ...design.utilities.keys("functional"),
];
const ownClasses = new Set(
  [
    ...files(join(root, "src"), /\.css$/),
    ...files(join(root, ".storybook"), /\.css$/),
  ]
    .flatMap((file) => [
      ...readFileSync(file, "utf8").matchAll(/\.(-?[_a-zA-Z][\w-]*)/g),
    ])
    .map((match) => match[1]),
);

const known = new Map();
function makesCss(candidate) {
  if (!known.has(candidate))
    known.set(candidate, design.candidatesToCss([candidate])[0] !== null);
  return known.get(candidate);
}

// The utility a class names, without its variants, `!` or negative sign.
function utilityOf(candidate) {
  return candidate
    .split(/:(?![^[]*\])/)
    .pop()
    .replace(/^!|!$/g, "")
    .replace(/^-/, "");
}

function looksLikeUtility(candidate) {
  const base = utilityOf(candidate);
  if (base.includes("[")) return false;
  return roots.some((name) => base === name || base.startsWith(`${name}-`));
}

// A class that makes no CSS on purpose: one of the stylesheets' own, or a
// marker another class refers to (`group`, `peer/name`).
function isOwnClass(candidate) {
  const base = utilityOf(candidate);
  return ownClasses.has(base) || /^(group|peer)(\/[\w-]+)?$/.test(base);
}

const problems = [];
for (const file of files(join(root, "src"), /\.tsx?$/)) {
  const source = readFileSync(file, "utf8");
  for (const match of source.matchAll(/"([^"\n]*)"|'([^'\n]*)'|`([^`]*)`/g)) {
    const text = (match[1] ?? match[2] ?? match[3]).replace(
      /\$\{[^}]*\}/g,
      " ",
    );
    const tokens = text.split(/\s+/).filter(Boolean);
    // A class list is classes only, at least one of them working, or it is
    // what a `className` is set to, where a word that is no class at all is
    // reported too; prose and ids have words that are not classes. A template
    // with a nested template in it is cut short by the pattern, so it is
    // never taken as a whole `className`.
    const isClassName =
      !match[0].includes("${") &&
      /[cC]lassName\s*[=:]\s*\{?\s*$/.test(
        source.slice(Math.max(0, match.index - 40), match.index),
      );
    const isClassList = tokens.every(
      (token) =>
        makesCss(token) || isOwnClass(token) || looksLikeUtility(token),
    );
    if (
      tokens.length === 0 ||
      !(isClassName || (isClassList && tokens.some(makesCss)))
    )
      continue;
    for (const token of tokens) {
      if (makesCss(token) || isOwnClass(token)) continue;
      const line = source.slice(0, match.index).split("\n").length;
      problems.push(`${relative(root, file)}:${line}: ${token}`);
    }
  }
}

if (problems.length > 0) {
  console.error(
    `Classes that make no CSS (a token renamed or removed from src/styles?):\n${problems.join("\n")}`,
  );
  process.exit(1);
}
console.log("Every Tailwind class in src makes CSS.");
