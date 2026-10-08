// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Checks the translations in `locales/` against the source language and the
// code that uses them. Failing: a syntax error; a message out of its file's
// namespace, with attributes or without a value; a translation with a message,
// file or placeholder the source lacks, or a term it does not define; a source
// message no code uses; an id in Rust's `t!` or Objective-C's
// `screenwide_osc_localized` the source lacks; out-of-date
// `src/i18n/messages.ts`. Untranslated messages are listed but pass: the
// source fills them in.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { extname, join, relative } from "node:path";
import process from "node:process";

import {
  languages,
  PSEUDO,
  readLanguage,
  SOURCE,
  TYPES_FILE,
  typesSource,
} from "./i18n-catalog.mjs";
import {
  INFO_PLIST,
  MACOS_DIR,
  macosFiles,
  PLIST_MESSAGES,
  sourceTexts,
} from "./i18n-macos.mjs";

const plistIds = new Set(Object.values(PLIST_MESSAGES));

const errors = [];

/** Every file below `directory` with one of `extensions`. */
const filesIn = (directory, extensions) =>
  readdirSync(directory, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && extensions.has(extname(entry.name)))
    .map((entry) => join(entry.parentPath, entry.name));

/** Rules every language follows, the source included. */
function checkShape(language, entries) {
  for (const [id, entry] of entries) {
    if (id.startsWith("-")) continue;
    if (!id.startsWith(`${entry.file}-`)) {
      errors.push(`${entry.at}: ${id} must start with "${entry.file}-"`);
    }
    if (!entry.hasValue) errors.push(`${entry.at}: ${id} has no value`);
    if (entry.attributes > 0) {
      errors.push(
        `${entry.at}: ${id} has attributes; give each string its own message`,
      );
    }
  }
  for (const [id, entry] of entries) {
    for (const term of entry.terms) {
      if (!entries.has(term)) {
        errors.push(
          `${entry.at}: ${id} uses ${term}, not defined in ${language}`,
        );
      }
    }
    for (const message of entry.messages) {
      if (!entries.has(message)) {
        errors.push(
          `${entry.at}: ${id} uses ${message}, not defined in ${language}`,
        );
      }
    }
  }
}

const source = readLanguage(SOURCE);
errors.push(...source.errors);
checkShape(SOURCE, source.entries);

const untranslated = [];
for (const language of languages()) {
  if (language === SOURCE) continue;
  if (language === PSEUDO) {
    errors.push(
      `${language} is the generated pseudo-locale; remove its folder`,
    );
    continue;
  }
  let canonical = null;
  try {
    [canonical] = Intl.getCanonicalLocales(language);
  } catch {
    // Reported below as not canonical.
  }
  if (canonical !== language) {
    errors.push(
      `${language} is not a canonical BCP 47 tag${canonical ? `; use ${canonical}` : ""}`,
    );
  }
  const translation = readLanguage(language);
  errors.push(...translation.errors);
  checkShape(language, translation.entries);
  for (const file of translation.files) {
    if (!source.files.includes(file)) {
      errors.push(`${language}/${file}.ftl has no counterpart in ${SOURCE}`);
    }
  }
  for (const [id, entry] of translation.entries) {
    const original = source.entries.get(id);
    if (!original) {
      if (!id.startsWith("-")) {
        errors.push(`${entry.at}: ${id} is not in ${SOURCE}`);
      }
      continue;
    }
    if (original.file !== entry.file) {
      errors.push(`${entry.at}: ${id} belongs in ${original.file}.ftl`);
    }
    for (const variable of entry.variables.keys()) {
      if (!original.variables.has(variable)) {
        errors.push(
          `${entry.at}: ${id} uses $${variable}, which ${SOURCE} never passes`,
        );
      }
    }
  }
  const missing = [...source.entries.keys()].filter(
    (id) => !id.startsWith("-") && !translation.entries.has(id),
  ).length;
  if (missing > 0)
    untranslated.push(`${language}: ${String(missing)} untranslated`);
}

// Usage. Ids are passed to `t`, `t!` and `screenwide_osc_localized` as
// literals, so a quoted id in the code is a use; messages may also include
// one another.
const typescript = filesIn("src", new Set([".ts", ".tsx"]))
  .filter((file) => file !== join(TYPES_FILE))
  .map((file) => readFileSync(file, "utf8"))
  .join("\n");
// Rust and the Objective-C overlays. Comments are left out: they may show
// the macro without using a message.
const native = filesIn("src-tauri/src", new Set([".rs", ".m", ".h"]))
  .flatMap((file) => readFileSync(file, "utf8").split("\n"))
  .filter((line) => !line.trimStart().startsWith("//"))
  .join("\n");
const referenced = new Set(
  [...source.entries.values()].flatMap(({ messages }) => [...messages]),
);
for (const [id, entry] of source.entries) {
  if (id.startsWith("-") || referenced.has(id) || plistIds.has(id)) continue;
  const quoted = `"${id}"`;
  if (!typescript.includes(quoted) && !native.includes(quoted)) {
    errors.push(`${entry.at}: ${id} is not used anywhere`);
  }
}
const nativeUses = native.matchAll(
  /(\bt!\(|\bscreenwide_osc_localized\(@)\s*"([^"]+)"/g,
);
for (const [, call, id] of nativeUses) {
  if (!source.entries.has(id) || id.startsWith("-")) {
    errors.push(`${call}"${id}") uses a message ${SOURCE} does not define`);
  }
}

if (source.errors.length === 0) {
  const expected = await typesSource(source.entries);
  if (readFileSync(TYPES_FILE, "utf8") !== expected) {
    errors.push(`${TYPES_FILE} is out of date; run pnpm i18n:generate`);
  }
}

// The macOS bundle: generated strings, and `Info.plist` holding the source
// text the strings files translate.
const plist = readFileSync(INFO_PLIST, "utf8");
for (const [key, text] of sourceTexts()) {
  if (!plist.includes(`<key>${key}</key>\n  <string>${text}</string>`)) {
    errors.push(`${INFO_PLIST}: ${key} must read "${text}", as in ${SOURCE}`);
  }
}
const expectedMacos = macosFiles();
const actualMacos = existsSync(MACOS_DIR)
  ? filesIn(MACOS_DIR, new Set([".strings"])).map((file) =>
      relative(MACOS_DIR, file),
    )
  : [];
const macosStale =
  actualMacos.length !== expectedMacos.size ||
  [...expectedMacos].some(
    ([path, contents]) =>
      !actualMacos.includes(path) ||
      readFileSync(join(MACOS_DIR, path), "utf8") !== contents,
  );
if (macosStale) {
  errors.push(`${MACOS_DIR} is out of date; run pnpm i18n:generate`);
}

for (const line of untranslated) console.log(line);
if (errors.length > 0) {
  console.error("Translation problems:");
  for (const error of errors) console.error(`- ${error}`);
  process.exitCode = 1;
}
