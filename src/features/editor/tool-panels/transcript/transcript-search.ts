// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TranscriptUnit, WordSelection } from "./transcript-units";

/** Text as a search compares it: no case, no accents, no punctuation, and
 * runs of spaces as one. Apostrophes stay, so "it's" and "its" differ. */
const foldForSearch = (text: string) =>
  text
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .replace(/[^\p{L}\p{N}\s']/gu, "")
    .toLowerCase()
    .replace(/\s+/gu, " ")
    .trim();

/** What a search match covers, and exactly where in the first and last
 * unit's own displayed text it starts and ends, so a replacement can keep
 * whatever of those two units sits outside the match. */
export type TranscriptMatch = WordSelection & {
  /** The index, in `units[last].text`, right after the match's last
   * character. */
  endOffset: number;
  /** The index, in `units[first].text`, of the match's first character. */
  startOffset: number;
};

/** One character of `text` folded on its own, so the folded string it
 * builds stays one character for one original character (stripping a
 * combining mark folds it into the base letter before it instead). */
const foldOneChar = (char: string) =>
  char
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .replace(/[^\p{L}\p{N}\s']/gu, "")
    .toLowerCase();

/** `text` folded for search, alongside the original index each folded
 * character came from: runs of whitespace collapse to one folded space, at
 * the first original index in the run, the way `foldForSearch` does for a
 * single unit's own text. */
function foldWithOffsets(text: string): { folded: string; offsets: number[] } {
  let folded = "";
  const offsets: number[] = [];
  let lastWasSpace = true;
  for (let at = 0; at < text.length; at++) {
    for (const out of foldOneChar(text[at])) {
      const isSpace = /\s/u.test(out);
      if (isSpace && lastWasSpace) continue;
      lastWasSpace = isSpace;
      folded += out;
      offsets.push(at);
    }
  }
  if (folded.endsWith(" ")) {
    folded = folded.slice(0, -1);
    offsets.pop();
  }
  return { folded, offsets };
}

/**
 * Every place `query` appears in the transcript's text as shown, corrections
 * included, as the units each one covers and exactly where it starts and
 * ends inside them. A match may begin or end inside a word; one may run
 * across several words. Matches never overlap.
 */
export function findInTranscript(
  units: readonly TranscriptUnit[],
  query: string,
): TranscriptMatch[] {
  const needle = foldForSearch(query);
  if (!needle) return [];
  let haystack = "";
  const owners: number[] = [];
  // The original index, in its owner's own text, of each haystack
  // character; unused for the space a separator between units adds, since
  // `needle` is trimmed and so a match never starts or ends on one.
  const originals: number[] = [];
  units.forEach((unit, index) => {
    const { folded, offsets } = foldWithOffsets(unit.text);
    if (!folded) return;
    if (haystack) {
      haystack += " ";
      owners.push(index);
      originals.push(0);
    }
    haystack += folded;
    for (const offset of offsets) {
      owners.push(index);
      originals.push(offset);
    }
  });
  const matches: TranscriptMatch[] = [];
  for (
    let at = haystack.indexOf(needle);
    at !== -1;
    at = haystack.indexOf(needle, at + needle.length)
  ) {
    const end = at + needle.length - 1;
    matches.push({
      endOffset: originals[end] + 1,
      first: owners[at],
      last: owners[end],
      startOffset: originals[at],
    });
  }
  return matches;
}
