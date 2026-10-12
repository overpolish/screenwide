// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The profanity lists in `profanity/`, one file a language, each loaded only
 * once a transcript in its language asks. They are written for speech:
 * swearing, insults and slurs, without the adult-content terms, everyday
 * words and filter-dodging spellings general blocklists carry.
 */
const LIST_FILES = import.meta.glob<string>("./profanity/*.txt", {
  import: "default",
  query: "?raw",
});

/** Languages the transcriber reports under another code than a list's:
 * Nynorsk shares Norwegian's, Cantonese the Chinese characters written for
 * it, and Malay Indonesian's. */
const SAME_LIST: Record<string, string> = { ms: "id", nn: "no", yue: "zh" };

/** Scripts written without spaces between words, whose entries are found
 * inside the text rather than word by word. */
const UNSPACED =
  /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Thai}]/u;

/** Text as profanity is matched: no case, and nothing but letters, marks
 * and digits, so "Damn!" and "damn" fold alike and an entry's own spacing is
 * kept apart by the caller. */
export const foldProfanityText = (text: string) =>
  text
    .normalize("NFC")
    .toLowerCase()
    .replace(/[^\p{L}\p{M}\p{N}]+/gu, "");

export type ProfanityLexicon = {
  /** Expressions of several words, by their first word, each as its words
   * folded. */
  phrases: ReadonlyMap<string, readonly (readonly string[])[]>;
  /** Expressions in a script written without spaces, folded. */
  unspaced: readonly string[];
  /** Single words, folded. */
  words: ReadonlySet<string>;
};

const fileFor = (code: string) =>
  LIST_FILES[`./profanity/${code}.txt`] as (() => Promise<string>) | undefined;

/** The lexicon of `lists`, each one expression a line; `#` starts a note. */
function lexiconOf(lists: readonly string[]): ProfanityLexicon {
  const words = new Set<string>();
  const phrases = new Map<string, string[][]>();
  const unspaced = new Set<string>();
  for (const list of lists) {
    for (const line of list.split("\n")) {
      const entry = line.trim();
      if (entry === "" || entry.startsWith("#")) continue;
      if (UNSPACED.test(entry)) {
        unspaced.add(foldProfanityText(entry));
        continue;
      }
      const parts = entry
        .split(/\s+/)
        .map(foldProfanityText)
        .filter((part) => part !== "");
      if (parts.length === 1) words.add(parts[0]);
      else if (parts.length > 1) {
        const starting = phrases.get(parts[0]) ?? [];
        starting.push(parts);
        phrases.set(parts[0], starting);
      }
    }
  }
  return { phrases, unspaced: [...unspaced], words };
}

const loaded = new Map<string, Promise<ProfanityLexicon>>();

/** The profanity to look for in a transcript in `language`: English's list
 * always, since English swearing turns up in every language, with the
 * language's own where it has one. Loaded once each. */
export function loadProfanity(
  language: string | null,
): Promise<ProfanityLexicon> {
  const code = language ? (SAME_LIST[language] ?? language) : null;
  const codes = ["en", ...(code && code !== "en" ? [code] : [])];
  const key = codes.join("+");
  const cached = loaded.get(key);
  if (cached) return cached;
  const loading = Promise.all(
    codes.map((name) => fileFor(name)?.() ?? Promise.resolve("")),
  ).then(lexiconOf);
  loaded.set(key, loading);
  return loading;
}
