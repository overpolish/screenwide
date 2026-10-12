// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fillerSoundsFor, foldFillerText } from "./transcript-filler-lists";

import type { TranscriptUnit, WordSelection } from "./transcript-units";

/** A word said again right after itself within this long still reads as
 * the speaker starting over, not two words. */
const REPEAT_WITHIN_MS = 1_000;

/**
 * Every filler the transcript still has kept, as the run of units it covers:
 * a hesitation sound in English or the transcript's `language`, and the
 * first of a word said twice within a second, as a speaker starting over
 * does. Words are read as the transcript shows them, so a word corrected
 * into a filler is one, and one corrected out of it is not. A word already
 * cut, or a pause, is never a filler. Fillers are in transcript order, and
 * each names the sound or word folded, so every one of a kind can be found
 * together.
 */
export function findFillerWords(
  units: readonly TranscriptUnit[],
  language: string | null,
): { folded: string; span: WordSelection }[] {
  const sounds = fillerSoundsFor(language);
  const kept = units
    .map((unit, index) => ({ index, unit }))
    .filter(({ unit }) => unit.kind === "word" && !unit.removed);
  const fillers: { folded: string; span: WordSelection }[] = [];
  kept.forEach(({ index, unit }, position) => {
    const folded = foldFillerText(unit.text);
    if (folded === "") return;
    const next = kept[position + 1] as (typeof kept)[number] | undefined;
    const isRepeat =
      next !== undefined &&
      foldFillerText(next.unit.text) === folded &&
      next.unit.startMs - unit.endMs < REPEAT_WITHIN_MS;
    if (sounds.has(folded) || isRepeat)
      fillers.push({ folded, span: { first: index, last: index } });
  });
  return fillers;
}
