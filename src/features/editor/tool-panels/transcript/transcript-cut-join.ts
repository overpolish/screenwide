// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TranscriptRange } from "../tool-panel-transcript";

/** A run of units chosen to cut, by index into the array it was found in. */
type UnitSpan = { first: number; last: number };

type CutUnit = TranscriptRange & { kind: "pause" | "word" };

/** What a cut of nothing but silence keeps of it: a beat where the voice
 * stops and starts again, so the words either side never run together. */
const PAUSE_CUT_KEEP_MS = 200;

const wordBefore = (units: readonly CutUnit[], index: number) => {
  for (let at = index - 1; at >= 0; at--)
    if (units[at].kind === "word") return units[at];
  return undefined;
};

const wordAfter = (units: readonly CutUnit[], index: number) => {
  for (let at = index + 1; at < units.length; at++)
    if (units[at].kind === "word") return units[at];
  return undefined;
};

/** A cut of only pauses: all but a beat of them, the beat left beside the
 * words, or beside the one word there is at the recording's start or end. */
function pauseCutRange(
  units: readonly CutUnit[],
  { first, last }: UnitSpan,
): TranscriptRange {
  const startMs = units[first].startMs;
  const endMs = units[last].endMs;
  const keep = Math.min(PAUSE_CUT_KEEP_MS, endMs - startMs);
  const hasBefore = wordBefore(units, first) !== undefined;
  const hasAfter = wordAfter(units, last) !== undefined;
  const keepBefore = !hasBefore ? 0 : hasAfter ? keep / 2 : keep;
  const keepAfter = !hasAfter ? 0 : hasBefore ? keep / 2 : keep;
  return { endMs: endMs - keepAfter, startMs: startMs + keepBefore };
}

/**
 * The stretch a cut of `units[span.first..span.last]` removes. The chosen
 * words go with the silence either side of them, and the join keeps one
 * pause, the longer of the two that were there: a word inside a sentence
 * leaves the words around it touching, the last words of a sentence leave
 * the break after it, a sentence between two others leaves one sentence
 * break, and nothing ever adds silence, stacks two pauses into one, or
 * reaches into a word that stays. The pause kept is split across the join as
 * evenly as the silence on each side allows. Before the first word and after
 * the last, the recording's own start and end stand in for the neighbour.
 */
export function joinedCutRange(
  units: readonly CutUnit[],
  span: UnitSpan,
  durationMs: number,
): TranscriptRange {
  let firstWord = -1;
  let lastWord = -1;
  for (let at = span.first; at <= span.last; at++) {
    if (units[at].kind !== "word") continue;
    if (firstWord < 0) firstWord = at;
    lastWord = at;
  }
  if (firstWord < 0) return pauseCutRange(units, span);
  const opening = units[firstWord].startMs;
  const closing = units[lastWord].endMs;
  const before = wordBefore(units, firstWord)?.endMs ?? 0;
  const after =
    wordAfter(units, lastWord)?.startMs ?? Math.max(durationMs, closing);
  const gapBefore = Math.max(0, opening - before);
  const gapAfter = Math.max(0, after - closing);
  const keep = Math.max(gapBefore, gapAfter);
  const keepAfter = Math.min(gapAfter, keep - Math.min(gapBefore, keep / 2));
  const keepBefore = keep - keepAfter;
  return { endMs: after - keepAfter, startMs: before + keepBefore };
}

/** Every span in `spans` turned into the stretch its cut removes, in the
 * order given. */
export function joinedCutRanges(
  units: readonly CutUnit[],
  spans: readonly UnitSpan[],
  durationMs: number,
): TranscriptRange[] {
  return spans.map((span) => joinedCutRange(units, span, durationMs));
}
