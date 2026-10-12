// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  foldProfanityText,
  ProfanityLexicon,
} from "./transcript-profanity-lists";

import type { TranscriptUnit, WordSelection } from "./transcript-units";

type Token = { text: string; unit: number };

/** Every word the transcript still keeps, as the folded words it shows,
 * each with the unit it belongs to: a corrected unit can hold several, and a
 * word corrected into profanity is profanity. */
const keptTokens = (units: readonly TranscriptUnit[]): Token[] =>
  units.flatMap((unit, index) =>
    unit.kind !== "word" || unit.removed
      ? []
      : unit.text
          .split(/\s+/)
          .map((word) => ({ text: foldProfanityText(word), unit: index }))
          .filter((token) => token.text !== ""),
  );

/** Where each entry written without spaces occurs in the kept words run
 * together, as the token runs it covers. */
function unspacedMatches(
  tokens: readonly Token[],
  entries: readonly string[],
): { first: number; last: number; text: string }[] {
  if (entries.length === 0) return [];
  let joined = "";
  const owner: number[] = [];
  tokens.forEach((token, position) => {
    joined += token.text;
    for (let index = 0; index < token.text.length; index++)
      owner.push(position);
  });
  const matches: { first: number; last: number; text: string }[] = [];
  for (const entry of entries) {
    for (
      let at = joined.indexOf(entry);
      at >= 0;
      at = joined.indexOf(entry, at + entry.length)
    )
      matches.push({
        first: owner[at],
        last: owner[at + entry.length - 1],
        text: entry,
      });
  }
  return matches;
}

/**
 * Every stretch of profanity the transcript still keeps, by the lists in
 * `lexicon`, leaving out what was marked "Not profanity" (`allowed`, as
 * folded text). A phrase is found across the units it spans, and a match in
 * a script written without spaces wherever it occurs in the words run
 * together. Each names its folded text, so every one alike can be found
 * together, and they are in transcript order without overlapping.
 */
export function findProfanity(
  units: readonly TranscriptUnit[],
  lexicon: ProfanityLexicon,
  allowed: ReadonlySet<string>,
): { folded: string; span: WordSelection }[] {
  const tokens = keptTokens(units);
  const found: { first: number; last: number; text: string }[] = [];
  tokens.forEach((token, position) => {
    if (lexicon.words.has(token.text))
      found.push({ first: position, last: position, text: token.text });
    for (const phrase of lexicon.phrases.get(token.text) ?? []) {
      const end = position + phrase.length - 1;
      if (
        end < tokens.length &&
        phrase.every((part, offset) => tokens[position + offset].text === part)
      )
        found.push({ first: position, last: end, text: phrase.join(" ") });
    }
  });
  found.push(...unspacedMatches(tokens, lexicon.unspaced));
  // The longest match at each start wins, and a match inside one already
  // taken is dropped, so "son of a bitch" is one finding, not two.
  found.sort((a, b) => a.first - b.first || b.last - a.last);
  const findings: { folded: string; span: WordSelection }[] = [];
  let reached = -1;
  for (const match of found) {
    if (match.first <= reached || allowed.has(match.text)) continue;
    reached = match.last;
    findings.push({
      folded: match.text,
      span: { first: tokens[match.first].unit, last: tokens[match.last].unit },
    });
  }
  return findings;
}
