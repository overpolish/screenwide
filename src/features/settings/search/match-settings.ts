// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { SettingsSearchEntry } from "./settings-search-index";

// How strongly a word typed matches a word in a setting, best first: the
// whole word, its start, its letters in order as an abbreviation ("ptr" for
// "pointer"), or a typo away.
const WHOLE = 1;
const START = 0.9;
const ABBREVIATION = 0.6;
const TYPO = 0.5;

// A match in the title counts most, then in the words a setting is also
// found by, then its description, then where it lives.
const FIELDS = [
  ["title", 1],
  ["keywords", 0.7],
  ["description", 0.5],
  ["location", 0.4],
] as const;

// Lower case, accents dropped, split into words of letters and digits.
const words = (text: string) =>
  text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .split(/[^\p{L}\p{N}]+/u)
    .filter(Boolean);

const isAbbreviation = (typed: string, word: string) => {
  if (typed[0] !== word[0]) return false;
  let at = 0;
  for (const letter of word) if (letter === typed[at]) at += 1;
  return at === typed.length;
};

// Edits apart, counting a swap of neighbours as one (optimal string
// alignment distance).
const distance = (a: string, b: string) => {
  const rows = Array.from({ length: a.length + 1 }, (_, i) =>
    Array.from({ length: b.length + 1 }, (_, j) =>
      i === 0 ? j : j === 0 ? i : 0,
    ),
  );
  for (let i = 1; i <= a.length; i += 1)
    for (let j = 1; j <= b.length; j += 1) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      rows[i][j] = Math.min(
        rows[i - 1][j] + 1,
        rows[i][j - 1] + 1,
        rows[i - 1][j - 1] + cost,
      );
      if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1])
        rows[i][j] = Math.min(rows[i][j], rows[i - 2][j - 2] + 1);
    }
  return rows[a.length][b.length];
};

const wordScore = (typed: string, word: string) => {
  if (word === typed) return WHOLE;
  if (word.startsWith(typed)) return START;
  if (typed.length >= 2 && isAbbreviation(typed, word)) return ABBREVIATION;
  // Short words are too easily a typo away from anything.
  if (typed.length < 4) return 0;
  const allowed = typed.length >= 7 ? 2 : 1;
  const closest = Math.min(
    distance(typed, word),
    distance(typed, word.slice(0, typed.length)),
  );
  return closest <= allowed ? TYPO : 0;
};

// The settings that match every word typed, best first; settings that score
// the same keep the panes' order.
export function matchSettings(
  entries: readonly SettingsSearchEntry[],
  query: string,
): SettingsSearchEntry[] {
  const typed = words(query);
  if (typed.length === 0) return [];
  const indexed = entries.map((entry) =>
    FIELDS.map(([field, weight]) => ({
      weight,
      words: words(entry[field] ?? ""),
    })),
  );
  return entries
    .map((entry, position) => {
      let score = 0;
      for (const word of typed) {
        let best = 0;
        for (const field of indexed[position])
          for (const candidate of field.words)
            best = Math.max(best, wordScore(word, candidate) * field.weight);
        if (best === 0) return null;
        score += best;
      }
      return { entry, position, score };
    })
    .filter((match) => match !== null)
    .sort((a, b) => b.score - a.score || a.position - b.position)
    .map(({ entry }) => entry);
}
