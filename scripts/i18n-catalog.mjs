// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Reads the Fluent files in `locales/` for `generate-i18n.mjs` and
// `check-i18n.mjs`: every message and term in a language, where it is, and
// the placeholders it takes.

import { readdirSync, readFileSync } from "node:fs";
import { basename, join } from "node:path";

import { parse } from "@fluent/syntax";
import { format, resolveConfig } from "prettier";

const LOCALES_DIR = "locales";
export const SOURCE = "en-US";
/** Generated from the source, never folders of their own. */
export const PSEUDO_LOCALES = ["en-XA", "en-XB"];
export const TYPES_FILE = "src/i18n/messages.ts";

const PLURAL_CATEGORIES = new Set(["few", "many", "one", "two", "zero"]);

export const languages = () =>
  readdirSync(LOCALES_DIR, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();

/** Calls `visit` on every syntax node below `node`, parents first. */
function walk(node, visit) {
  if (Array.isArray(node)) {
    for (const child of node) walk(child, visit);
    return;
  }
  if (!node || typeof node !== "object") return;
  if (typeof node.type === "string") visit(node);
  for (const [key, child] of Object.entries(node)) {
    if (key !== "span") walk(child, visit);
  }
}

/**
 * The placeholders a message takes, each typed `number` where it picks a
 * plural form or goes through `NUMBER()`, so a caller cannot pass a number
 * already written as text, which would always take the default variant.
 * The rest take text or a number; the bundle writes numbers in the app's
 * format locale.
 */
function referencesOf(entry) {
  const variables = new Map();
  const messages = new Set();
  const terms = new Set();
  walk(entry, (node) => {
    if (node.type === "SelectExpression") {
      const counts = node.variants.some(
        ({ key }) =>
          key.type === "NumberLiteral" || PLURAL_CATEGORIES.has(key.name),
      );
      if (counts && node.selector.type === "VariableReference") {
        variables.set(node.selector.id.name, "number");
      }
    } else if (node.type === "FunctionReference" && node.id.name === "NUMBER") {
      for (const argument of node.arguments.positional) {
        if (argument.type === "VariableReference") {
          variables.set(argument.id.name, "number");
        }
      }
    } else if (node.type === "VariableReference") {
      if (!variables.has(node.id.name)) {
        variables.set(node.id.name, "string | number");
      }
    } else if (node.type === "MessageReference") {
      messages.add(node.id.name);
    } else if (node.type === "TermReference") {
      terms.add(`-${node.id.name}`);
    }
  });
  return { messages, terms, variables };
}

/**
 * Every message and term in `language`, keyed by id (terms keep their `-`),
 * and the syntax errors and repeated ids found on the way.
 */
export function readLanguage(language) {
  const entries = new Map();
  const errors = [];
  const directory = join(LOCALES_DIR, language);
  const files = readdirSync(directory)
    .filter((file) => file.endsWith(".ftl"))
    .sort();
  for (const file of files) {
    const path = join(directory, file);
    const source = readFileSync(path, "utf8");
    const lineOf = (offset) => source.slice(0, offset).split("\n").length;
    for (const entry of parse(source, { withSpans: true }).body) {
      // An entry's span starts at its comment; its id is the line to edit.
      const start = (entry.id ?? entry).span.start;
      const at = `${path}:${String(lineOf(start))}`;
      if (entry.type === "Junk") {
        const reasons = entry.annotations.map(({ message }) => message);
        errors.push(`${at}: ${reasons.join("; ")}`);
        continue;
      }
      if (entry.type !== "Message" && entry.type !== "Term") continue;
      const id = `${entry.type === "Term" ? "-" : ""}${entry.id.name}`;
      const existing = entries.get(id);
      if (existing) {
        errors.push(`${at}: ${id} is already defined at ${existing.at}`);
        continue;
      }
      entries.set(id, {
        at,
        attributes: entry.attributes.length,
        file: basename(file, ".ftl"),
        hasValue: entry.value !== null,
        ...referencesOf(entry),
      });
    }
  }
  return {
    entries,
    errors,
    files: files.map((file) => basename(file, ".ftl")),
  };
}

/** `src/i18n/messages.ts` for the source language's messages. */
export async function typesSource(entries) {
  const fields = [...entries]
    .filter(([id]) => !id.startsWith("-"))
    .sort(([left], [right]) => (left < right ? -1 : 1))
    .map(([id, { variables }]) => {
      if (variables.size === 0) return `"${id}": null;`;
      const args = [...variables]
        .sort(([left], [right]) => (left < right ? -1 : 1))
        .map(([name, type]) => `${name}: ${type};`);
      return `"${id}": { ${args.join(" ")} };`;
    });
  const source = [
    "// SPDX-FileCopyrightText: 2026 overpolish",
    "// SPDX-License-Identifier: GPL-3.0-or-later",
    "",
    `// Generated from ${LOCALES_DIR}/${SOURCE} by \`pnpm i18n:generate\`. Do not edit.`,
    "",
    "/** Every message, with the placeholders it takes; `null` for none. */",
    `export type Messages = { ${fields.join(" ")} };`,
    "",
  ].join("\n");
  const options = await resolveConfig(TYPES_FILE);
  return format(source, { ...options, filepath: TYPES_FILE });
}
