// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TranscriptMove, TranscriptRange } from "../tool-panel-transcript";

import { TranscriptUnit, WordSelection } from "./transcript-units";
import { middleOf } from "./transcript-words";

/** The transcript as it plays, beside the transcript as it was heard. */
export type TranscriptOrder = {
  /** The units in the order they were heard. */
  heard: readonly TranscriptUnit[];
  /** Each unit's index among `heard`. */
  heardIndex: readonly number[];
  /** Each heard unit's index among `units`. */
  shownIndex: readonly number[];
  /** The units in the order the timeline plays them. */
  units: readonly TranscriptUnit[];
};

/** Which of `played` a unit plays in: the stretch holding its middle, or
 * for one the timeline cut away, the stretch it would be restored after,
 * the one ending latest before it, and the earliest in the recording where
 * none does. */
const stretchOf = (
  played: readonly TranscriptRange[],
  unit: TranscriptUnit,
) => {
  const middle = middleOf(unit);
  const holding = played.findIndex(
    (stretch) => stretch.startMs <= middle && middle < stretch.endMs,
  );
  if (holding >= 0) return holding;
  let before = -1;
  let earliest = -1;
  played.forEach((stretch, index) => {
    if (
      stretch.endMs <= middle &&
      (before < 0 || stretch.endMs > played[before].endMs)
    )
      before = index;
    if (earliest < 0 || stretch.startMs < played[earliest].startMs)
      earliest = index;
  });
  return before >= 0 ? before : earliest;
};

/**
 * `heard`, the units in the order they were heard, in the order the
 * timeline plays them, `played` being the stretches of the recording it
 * plays in that order. Words cut away stay where they would be restored, so
 * they read in place. With nothing played the order is as heard.
 */
export function transcriptInPlaybackOrder(
  heard: readonly TranscriptUnit[],
  played: readonly TranscriptRange[],
): TranscriptOrder {
  const heardIndex = heard.map((_, index) => index);
  if (played.length > 0) {
    const stretch = heard.map((unit) => stretchOf(played, unit));
    // Within one stretch, the order heard is the order played.
    heardIndex.sort((a, b) => stretch[a] - stretch[b] || a - b);
  }
  const shownIndex: number[] = [];
  heardIndex.forEach((at, index) => {
    shownIndex[at] = index;
  });
  return {
    heard,
    heardIndex,
    shownIndex,
    units: heardIndex.map((at) => heard[at]),
  };
}

/** The units `span` shows, as runs of the units heard one after another, in
 * the order heard: one run where nothing in it was moved apart. */
export function heardRuns(
  { heardIndex }: TranscriptOrder,
  { first, last }: WordSelection,
): WordSelection[] {
  const heard = heardIndex.slice(first, last + 1).sort((a, b) => a - b);
  const runs: WordSelection[] = [];
  for (const at of heard) {
    const run = runs[runs.length - 1] as WordSelection | undefined;
    if (run && at === run.last + 1) run.last = at;
    else runs.push({ first: at, last: at });
  }
  return runs;
}

/** Whether the units `span` shows read as they were heard, one after
 * another: only then are they one stretch of the recording, which a
 * correction or a highlight can name. */
export const isHeardInOrder = (
  { heardIndex }: TranscriptOrder,
  { first, last }: WordSelection,
) => {
  for (let index = first; index < last; index++)
    if (heardIndex[index + 1] !== heardIndex[index] + 1) return false;
  return true;
};

/** The heard units `span` shows from first to last, which `isHeardInOrder`
 * says are one stretch. */
export const heardSpanOf = (
  { heardIndex }: TranscriptOrder,
  { first, last }: WordSelection,
): WordSelection => ({ first: heardIndex[first], last: heardIndex[last] });

/** Heard unit indices `indices` as the indices they show at. */
export const shownIndices = (
  { shownIndex }: TranscriptOrder,
  indices: Iterable<number>,
): Set<number> => new Set([...indices].map((at) => shownIndex[at]));

/**
 * The footage dragging the units `span` shows to stand before the unit
 * shown at `before` moves: from the start of the first unit it keeps to
 * the end of the last, and on to the next word where that follows it with
 * a space too short to show as a pause, so the words keep the space after
 * them and land as said. It lands just before the start of the unit at
 * `before`, or at the end for the unit count. Null where the span keeps
 * nothing to move.
 */
export function movedFootage(
  order: TranscriptOrder,
  span: WordSelection,
  before: number,
): TranscriptMove | null {
  const { heard, heardIndex, shownIndex, units } = order;
  let first = -1;
  let last = -1;
  for (let index = span.first; index <= span.last; index++) {
    if (units[index].removed) continue;
    if (first < 0) first = index;
    last = index;
  }
  if (first < 0) return null;
  const end = units[last];
  const nextHeard = heardIndex[last] + 1;
  const next = heard[nextHeard] as TranscriptUnit | undefined;
  const keepsSpace =
    end.kind === "word" &&
    next?.kind === "word" &&
    !next.removed &&
    shownIndex[nextHeard] === last + 1;
  return {
    beforeMs: before < units.length ? units[before].startMs : null,
    endMs: keepsSpace ? next.startMs : end.endMs,
    startMs: units[first].startMs,
  };
}
