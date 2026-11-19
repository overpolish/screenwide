// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type ScriptText } from "../script-words";

import { wordAt } from "./cue-anchors";

/** How many words before its own a cue shows: a glance ahead, so it is
 * seen before its line is said rather than as it is. */
export const LEAD_WORDS = 6;

/** A run of spaces holding an empty line, which ends a paragraph. */
const PARAGRAPH_BREAK = /\n[^\S\n]*\n/;

/** The index of the first word of the paragraph word `index` is in. */
function paragraphStart(script: ScriptText, index: number) {
  let start = 0;
  let word = -1;
  for (const piece of script.pieces) {
    if (piece.kind === "word") {
      word = piece.index;
      if (word >= index) return start;
    } else if (PARAGRAPH_BREAK.test(piece.text)) start = word + 1;
  }
  return start;
}

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
 * read: the last whose word has been reached, until its paragraph has been
 * read or the next cue's word is reached. With none showing, the next cue
 * shows a few words before its own, a glance ahead, but never before its
 * paragraph; it never takes over early from one being read.
 */
export function shownCue<T extends { at: number }>(
  cues: readonly T[],
  script: ScriptText,
  place: number,
): T | null {
  const word = Math.floor(place);
  let reached: { cue: T; index: number } | null = null;
  let coming: { cue: T; index: number } | null = null;
  for (const cue of cues) {
    const index = wordAt(script, cue.at);
    if (index < 0) continue;
    if (index <= word) {
      if (!reached || index >= reached.index) reached = { cue, index };
      continue;
    }
    const from = Math.max(index - LEAD_WORDS, paragraphStart(script, index));
    if (word >= from && (!coming || index < coming.index))
      coming = { cue, index };
  }
  if (reached && word < paragraphEnd(script, reached.index)) return reached.cue;
  return coming?.cue ?? null;
}
