// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Writes what the build needs from `locales/`: `src/i18n/messages.ts`, so `t`
// accepts only message ids that exist, with the placeholders each one takes;
// and the macOS bundle's `InfoPlist.strings` per language. Run after adding
// a language, or adding, renaming or removing a message, or changing its
// placeholders.

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import process from "node:process";

import {
  readLanguage,
  SOURCE,
  TYPES_FILE,
  typesSource,
} from "./i18n-catalog.mjs";
import { MACOS_DIR, macosFiles } from "./i18n-macos.mjs";

const { entries, errors } = readLanguage(SOURCE);
if (errors.length > 0) {
  for (const error of errors) console.error(error);
  process.exit(1);
}
writeFileSync(TYPES_FILE, await typesSource(entries));

// Rewritten whole, so a removed language leaves no folder behind.
rmSync(MACOS_DIR, { force: true, recursive: true });
for (const [path, contents] of macosFiles()) {
  const target = join(MACOS_DIR, path);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, contents);
}
