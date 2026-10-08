// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FluentBundle, FluentResource } from "@fluent/bundle";
import { invoke } from "@tauri-apps/api/core";
import { isRTL } from "react-aria";

import { isWindows } from "../lib/platform";

import { accented, bidi } from "./pseudo";

import type { Messages } from "./messages";
import type { AppLocale } from "../bindings/AppLocale";

/** The language every message is written in first, and what a message a
 * translation lacks falls back to. Matches `SOURCE` in `src-tauri/src/i18n.rs`. */
const SOURCE = "en-US";
/** Accented, lengthened English for spotting text that bypasses translation
 * and layouts that break under longer words. */
export const PSEUDO = "en-XA";
/** English read backwards and laid out right to left, for spotting text that
 * bypasses translation and layouts that do not mirror. */
export const PSEUDO_BIDI = "en-XB";

export type Direction = "ltr" | "rtl";

const directionOf = (language: string): Direction =>
  language === PSEUDO_BIDI || isRTL(language) ? "rtl" : "ltr";

/** React Aria reads the writing direction from its locale alone. Where the
 * direction is right to left but the format locale reads left to right, it
 * is handed Arabic with Latin digits instead. */
const RIGHT_TO_LEFT_STAND_IN = "ar-u-nu-latn";

// The source is in every window's bundle, since every translation falls back
// to it; the rest load only in a window that uses them.
const sourceFiles = Object.values(
  import.meta.glob<string>("/locales/en-US/*.ftl", {
    eager: true,
    import: "default",
    query: "?raw",
  }),
);
const translationFiles = import.meta.glob<string>(
  ["/locales/*/*.ftl", "!/locales/en-US/*.ftl"],
  { import: "default", query: "?raw" },
);

const languageOf = (path: string) => path.split("/")[2];

/** Every translation there is, the source first. */
export const LANGUAGES = [
  SOURCE,
  ...new Set(Object.keys(translationFiles).map(languageOf)),
];

export type MessageId = keyof Messages;
type MessageArgs<Id extends MessageId> = Messages[Id] extends null
  ? []
  : [args: Messages[Id]];

/** A file that fails to parse still contributes the messages around its
 * error, so one typo in a translation costs one message. */
const createBundle = (
  locale: string,
  sources: readonly string[],
  transform?: (text: string) => string,
) => {
  const bundle = new FluentBundle(locale, { transform });
  for (const source of sources) {
    for (const error of bundle.addResource(new FluentResource(source))) {
      console.error(`Translation for ${locale}:`, error);
    }
  }
  return bundle;
};

const sourceBundle = createBundle(SOURCE, sourceFiles);

let active: {
  bundle: FluentBundle;
  /** The source, while the translation is another language. */
  fallback: FluentBundle | null;
  locale: AppLocale;
} = {
  bundle: sourceBundle,
  fallback: null,
  locale: { formatLocale: SOURCE, language: SOURCE },
};

/**
 * Switches every later `t` call to `locale`'s language. Numbers in messages,
 * and plural choices, follow its format locale, which Rust keeps in the same
 * language. A language with no translation falls back to the source.
 */
export async function loadLocale(locale: AppLocale) {
  const { formatLocale, language } = locale;
  const loaders = Object.entries(translationFiles)
    .filter(([path]) => languageOf(path) === language)
    .map(([, load]) => load());
  if (language === PSEUDO || language === PSEUDO_BIDI) {
    active = {
      bundle: createBundle(
        SOURCE,
        sourceFiles,
        language === PSEUDO ? accented : bidi,
      ),
      fallback: null,
      locale,
    };
  } else if (loaders.length === 0) {
    active = {
      bundle: sourceBundle,
      fallback: null,
      locale: {
        formatLocale: language === SOURCE ? formatLocale : SOURCE,
        language: SOURCE,
      },
    };
  } else {
    active = {
      bundle: createBundle(formatLocale, await Promise.all(loaders)),
      fallback: sourceBundle,
      locale,
    };
  }
  document.documentElement.lang = active.locale.language;
  applyDirection(appDirection());
}

/** The way the app's language is written. */
export const appDirection = () => directionOf(active.locale.language);

/** Lays the page out in `direction`: popovers and tooltips, rendered at the
 * end of the body, follow it too. */
export const applyDirection = (direction: Direction) => {
  document.documentElement.dir = direction;
};

/**
 * The locale for React Aria's `I18nProvider`, which lays out and formats by
 * it: the format locale, unless it reads the wrong way for `direction`, as
 * under the bidi pseudo-locale or Storybook's direction override.
 */
export const layoutLocale = (direction: Direction = appDirection()) => {
  const { formatLocale } = active.locale;
  return direction === "rtl" && !isRTL(formatLocale)
    ? RIGHT_TO_LEFT_STAND_IN
    : formatLocale;
};

/**
 * Loads the language Rust chose for the app, so web and native text agree.
 * Outside the desktop app there is no Rust to ask, and the source stays.
 */
export const loadAppLocale = () =>
  invoke<AppLocale>("get_app_locale").then(loadLocale, () => undefined);

/** The language in use, and the locale dates and numbers are written in. */
export const appLocale = () => active.locale;

/**
 * The message `id` in the app's language, with `args` in its placeholders.
 * Call it while rendering, not at module scope, so the text follows a
 * language loaded after import, as Storybook's locale switch does.
 */
export function t<Id extends MessageId>(
  id: Id,
  ...[args]: MessageArgs<Id>
): string {
  for (const bundle of [active.bundle, active.fallback]) {
    const message = bundle?.getMessage(id);
    if (!bundle || !message?.value) continue;
    const errors: Error[] = [];
    const text = bundle.formatPattern(message.value, args, errors);
    for (const error of errors) console.error(`Message ${id}:`, error);
    return text;
  }
  console.error(`No message ${id}`);
  return id;
}

/** What messages that differ by platform select on, as `$platform`. */
export const platformArg = () => (isWindows() ? "windows" : "macos");
