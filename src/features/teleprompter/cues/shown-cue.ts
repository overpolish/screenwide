// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type ScriptText } from "../script-words";

import { wordAt } from "./cue-anchors";

/** How many words before its own a cue shows: a glance ahead, so it is
 * seen before its line is said rather than as it is. */
export const LEAD_WORDS = 6;

/** A run of spaces holding an empty line, which ends a paragraph. */
const PARAGRAPH_BREAK = /\n[^\S\n]*\n/;

/** The index of the first word of the paragraph after the one word `index`
 * is in; past the last word for the last paragraph. */
function paragraphEnd(script: ScriptText, index: number) {
  let word = -1;
  for (const piece of script.pieces) {
    if (piece.kind === "word") word = piece.index;
    else if (word >= index && PARAGRAPH_BREAK.test(piece.text)) return word + 1;
  }
  return script.spoken.length;
}

/**
 * The cue to show with the reader at `place`, the index of the next word to
 * read: the last whose showing has begun, a few words before its own,
 * until its paragraph has been read or the next cue's showing begins; none
 * between.
 */
export function shownCue<T extends { at: number }>(
  cues: readonly T[],
  script: ScriptText,
  place: number,
): T | null {
  const word = Math.floor(place);
  let shown: { cue: T; index: number } | null = null;
  for (const cue of cues) {
    const index = wordAt(script, cue.at);
    if (index < 0 || word < index - LEAD_WORDS) continue;
    if (!shown || index >= shown.index) shown = { cue, index };
  }
  return shown && word < paragraphEnd(script, shown.index) ? shown.cue : null;
}
