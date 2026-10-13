// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TranscriptRange } from "../tool-panel-transcript";
import { middleOf } from "../transcript/transcript-words";

import type { CaptionBreak } from "../../../../bindings/CaptionBreak";
import type { CaptionBreakKind } from "../../../../bindings/CaptionBreakKind";
import type { CaptionWordChoice } from "../../../../bindings/CaptionWordChoice";
import type { CaptionWordKind } from "../../../../bindings/CaptionWordKind";

// Breaks and choices sit at moments as shares of the recording, so they
// name the same gap and the same word however a word's edges settle; these
// are the rules `screenwide_captions` reads them by.

/** The gap between two words, in a recording `durationMs` long. */
export type CaptionGapAt = {
  after: TranscriptRange;
  before: TranscriptRange;
  durationMs: number;
};

/** The shares of the recording a break in `gap` may sit in, after the
 * middle of the word before and up to the middle of the word after; null
 * where the two were not said one after the other. */
const sharesOf = ({ after, before, durationMs }: CaptionGapAt) => {
  const from = middleOf(before) / durationMs;
  const to = middleOf(after) / durationMs;
  return durationMs > 0 && from < to ? { from, to } : null;
};

const isIn = (shares: { from: number; to: number }, placed: CaptionBreak) =>
  shares.from < placed.sourceAt && placed.sourceAt <= shares.to;

/** Whether a break can be placed by hand in `gap`. */
export const canBreakIn = (gap: CaptionGapAt) => sharesOf(gap) !== null;

/** The break placed by hand in `gap`, if any. */
export function breakIn(breaks: readonly CaptionBreak[], gap: CaptionGapAt) {
  const shares = sharesOf(gap);
  return shares
    ? (breaks.find((placed) => isIn(shares, placed)) ?? null)
    : null;
}

/** `breaks` with a break of `kind` placed in `gap`, halfway between its
 * words, or with none placed there by hand for null. */
export function withBreak(
  breaks: readonly CaptionBreak[],
  gap: CaptionGapAt,
  kind: CaptionBreakKind | null,
): CaptionBreak[] {
  const shares = sharesOf(gap);
  if (!shares) return [...breaks];
  const kept = breaks.filter((placed) => !isIn(shares, placed));
  if (!kind) return kept;
  const halfway = (gap.before.endMs + gap.after.startMs) / 2 / gap.durationMs;
  const sourceAt = Math.min(Math.max(halfway, shares.from), shares.to);
  return [...kept, { kind, sourceAt }].sort((a, b) => a.sourceAt - b.sourceAt);
}

/** `choices` with the word at `word` chosen as `kind`, at its middle, or
 * left to the rules with null. */
export function withChoice(
  choices: readonly CaptionWordChoice[],
  { durationMs, word }: { durationMs: number; word: TranscriptRange },
  kind: CaptionWordKind | null,
): CaptionWordChoice[] {
  const kept = choices.filter(
    (choice) =>
      choice.sourceAt < word.startMs / durationMs ||
      choice.sourceAt > word.endMs / durationMs,
  );
  if (!kind || durationMs <= 0) return kept;
  return [...kept, { kind, sourceAt: middleOf(word) / durationMs }].sort(
    (a, b) => a.sourceAt - b.sourceAt,
  );
}
