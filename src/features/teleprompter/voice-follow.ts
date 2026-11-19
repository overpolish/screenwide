// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { spokenForm } from "./script-words";

// Following the voice: the last few words heard are found in the script
// near where the reader is, and the place moves on to the word after them.
// Speech recognition mishears, and readers skip, repeat and stumble, so the
// match is loose: the heard words are lined up with the script's as a
// common subsequence, a word allowed to be one letter out, and the last
// word heard, still settling, may be the start of one. A place far ahead,
// or behind, has to be matched by more words than one close by, so a word
// that comes up again later in the script does not pull the place there.

/** How many of the last words heard are matched. */
const TAIL = 6;
/** How far ahead of the place, and behind it, a match is looked for, in
 * words. */
const AHEAD = 60;
const BEHIND = 12;
/** A word matched on its own moves the place at most this many words;
 * further ahead takes two lined up, and beyond `NEAR_AHEAD`, or any way
 * behind, `FAR_MATCH`. */
const STEP = 3;
const NEAR_AHEAD = 12;
const FAR_MATCH = 3;

/** At most one letter added, taken away or changed. */
const withinOneEdit = (a: string, b: string) => {
  if (Math.abs(a.length - b.length) > 1) return false;
  let i = 0;
  let j = 0;
  let edits = 0;
  while (i < a.length && j < b.length) {
    if (a[i] === b[j]) {
      i += 1;
      j += 1;
      continue;
    }
    edits += 1;
    if (edits > 1) return false;
    if (a.length > b.length) i += 1;
    else if (b.length > a.length) j += 1;
    else {
      i += 1;
      j += 1;
    }
  }
  return edits + (a.length - i) + (b.length - j) <= 1;
};

const sameWord = (heard: string, script: string) =>
  heard === script ||
  (heard.length >= 4 && script.length >= 4 && withinOneEdit(heard, script));

/** How many of `heard` line up, in order, with `script`. */
const lined = (heard: string[], script: string[]) => {
  const row = new Array<number>(script.length + 1).fill(0);
  for (const word of heard) {
    let diagonal = 0;
    for (let j = 1; j <= script.length; j += 1) {
      const above = row[j];
      row[j] = sameWord(word, script[j - 1])
        ? diagonal + 1
        : Math.max(row[j], row[j - 1]);
      diagonal = above;
    }
  }
  return row[script.length];
};

/**
 * Where the reader is in `script`, its words in their spoken form, given
 * every word `heard` so far and the place `current`, the index of the next
 * word to read: the word after the best match of the last words heard, or
 * `current` where none matches well enough.
 */
export function followedPlace(
  script: readonly string[],
  heard: readonly string[],
  current: number,
): number {
  const tail = heard
    .map(spokenForm)
    .filter((word) => word !== "")
    .slice(-TAIL);
  if (tail.length === 0 || script.length === 0) return current;
  const last = tail[tail.length - 1];
  const from = Math.max(0, current - BEHIND);
  const to = Math.min(script.length - 1, current + AHEAD);
  let best: { end: number; score: number } | null = null;
  for (let end = from; end <= to; end += 1) {
    const word = script[end];
    // The match ends on the last word heard, which may be still settling
    // and so only begun.
    if (!sameWord(last, word) && !(last.length >= 3 && word.startsWith(last)))
      continue;
    const start = Math.max(0, end - tail.length - 1);
    const score = lined(tail, script.slice(start, end + 1));
    const ahead = end + 1 - current;
    const needed =
      ahead < 0 || ahead > NEAR_AHEAD ? FAR_MATCH : ahead > STEP ? 2 : 1;
    if (score < needed) continue;
    const distance = Math.abs(end + 1 - current);
    if (
      !best ||
      score > best.score ||
      (score === best.score && distance < Math.abs(best.end + 1 - current))
    )
      best = { end, score };
  }
  return best ? best.end + 1 : current;
}
