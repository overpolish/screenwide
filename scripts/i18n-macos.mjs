// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The macOS permission prompts, which the system reads from the app bundle
// rather than asking the app: `Info.plist` holds the source language's text,
// and one `<language>.lproj/InfoPlist.strings` per translation holds the rest.
// The folders also tell macOS which languages the app speaks, so the system's
// own dialogs, such as the file picker, follow the app's language.

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { FluentBundle, FluentResource } from "@fluent/bundle";

import { languages, PSEUDO_LOCALES, SOURCE } from "./i18n-catalog.mjs";

/** Copied into the bundle's `Contents/Resources` by `tauri.conf.json`. */
export const MACOS_DIR = "src-tauri/macos-localizations";
export const INFO_PLIST = "src-tauri/Info.plist";

/** Each localized `Info.plist` key and the message that fills it. */
export const PLIST_MESSAGES = {
  NSAudioCaptureUsageDescription: "macos-audio-capture-usage",
  NSCameraUsageDescription: "macos-camera-usage",
  NSMicrophoneUsageDescription: "macos-microphone-usage",
};

/** One language's messages as macOS sees them: no direction isolates. */
function bundleOf(language) {
  const bundle = new FluentBundle(language, { useIsolating: false });
  const path = join("locales", language, "macos.ftl");
  try {
    bundle.addResource(new FluentResource(readFileSync(path, "utf8")));
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  return bundle;
}

const textOf = (bundle, id) => {
  const message = bundle.getMessage(id);
  return message?.value ? bundle.formatPattern(message.value) : undefined;
};

/** The source language's text for each key, as `Info.plist` must hold it. */
export function sourceTexts() {
  const bundle = bundleOf(SOURCE);
  return Object.entries(PLIST_MESSAGES).map(([key, id]) => [
    key,
    textOf(bundle, id),
  ]);
}

/**
 * The folder name macOS matches a language by. The source language goes in
 * `en.lproj`, so every English variant finds it, as the bundle's development
 * region expects.
 */
const folderOf = (language) =>
  `${language === SOURCE ? language.split("-")[0] : language}.lproj`;

const quoted = (text) =>
  `"${text.replaceAll("\\", "\\\\").replaceAll('"', '\\"').replaceAll("\n", "\\n")}"`;

/**
 * Every file below `MACOS_DIR`, keyed by its path there. A message a
 * translation lacks falls back to the source language, as in the app.
 */
export function macosFiles() {
  const source = bundleOf(SOURCE);
  const files = new Map();
  for (const language of languages()) {
    if (PSEUDO_LOCALES.includes(language)) continue;
    const bundle = bundleOf(language);
    const lines = Object.entries(PLIST_MESSAGES).map(([key, id]) => {
      const text = textOf(bundle, id) ?? textOf(source, id) ?? "";
      return `${quoted(key)} = ${quoted(text)};`;
    });
    files.set(
      join(folderOf(language), "InfoPlist.strings"),
      [
        "/* SPDX-FileCopyrightText: 2026 overpolish */",
        "/* SPDX-License-Identifier: GPL-3.0-or-later */",
        "",
        `/* Generated from locales/${language}/macos.ftl by \`pnpm i18n:generate\`. Do not edit. */`,
        "",
        ...lines,
        "",
      ].join("\n"),
    );
  }
  return files;
}
