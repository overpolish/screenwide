// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { appLocale } from "../../../i18n/i18n";

/** The languages Whisper transcribes well, by the codes it takes: the list
 * OpenAI publishes for it, out of the hundred or so it knows. */
const TRANSCRIPTION_LANGUAGE_CODES = [
  "af",
  "ar",
  "hy",
  "az",
  "be",
  "bs",
  "bg",
  "ca",
  "zh",
  "hr",
  "cs",
  "da",
  "nl",
  "en",
  "et",
  "fi",
  "fr",
  "gl",
  "de",
  "el",
  "he",
  "hi",
  "hu",
  "is",
  "id",
  "it",
  "ja",
  "kn",
  "kk",
  "ko",
  "lv",
  "lt",
  "mk",
  "ms",
  "mr",
  "mi",
  "ne",
  "no",
  "fa",
  "pl",
  "pt",
  "ro",
  "ru",
  "sr",
  "sk",
  "sl",
  "es",
  "sw",
  "sv",
  "tl",
  "ta",
  "th",
  "tr",
  "uk",
  "ur",
  "vi",
  "cy",
];

/** `code`'s name in the app's language, or the code itself where the
 * platform has no name for it. */
export const transcriptionLanguageName = (code: string) => {
  try {
    return (
      new Intl.DisplayNames([appLocale().language], { type: "language" }).of(
        code,
      ) ?? code
    );
  } catch {
    return code;
  }
};

/** Every language on offer, named in the app's language and sorted by that
 * name. */
export const transcriptionLanguages = () => {
  const collator = new Intl.Collator(appLocale().formatLocale);
  return TRANSCRIPTION_LANGUAGE_CODES.map((code) => ({
    code,
    name: transcriptionLanguageName(code),
  })).sort((left, right) => collator.compare(left.name, right.name));
};
