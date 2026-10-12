// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { hasKeyboardSegmentEdits } from "./recording-timeline-cuts";
import {
  RecordingTimelineEdit,
  RecordingTimelineSegment,
} from "./recording-timeline-edit";
import {
  earliestSourceIndex,
  sourcePredecessorIndex,
} from "./recording-timeline-source-order";

/** A cut recorded in the edit, in shares of the whole recording, with the
 * rate it played at before it was cut. Remove silences and the Transcript
 * panel record theirs the same way. */
export type SourceCut = {
  playbackRate: number;
  sourceEnd: number;
  sourceStart: number;
};

type Range = { end: number; start: number };

/** Differences below this are the same place, as floating-point shares of a
 * recording can drift a little while splitting and rejoining. */
const CUT_EPSILON = 1e-9;

/** What of `range` no segment keeps, in source order. */
export const removedParts = (
  segments: readonly RecordingTimelineSegment[],
  range: Range,
): Range[] => {
  const parts: Range[] = [];
  let from = range.start;
  const inSourceOrder = [...segments].sort(
    (a, b) => a.sourceStart - b.sourceStart,
  );
  for (const segment of inSourceOrder) {
    if (segment.sourceEnd <= from) continue;
    if (segment.sourceStart >= range.end) break;
    if (segment.sourceStart - from > CUT_EPSILON)
      parts.push({ end: segment.sourceStart, start: from });
    from = Math.max(from, segment.sourceEnd);
  }
  if (range.end - from > CUT_EPSILON)
    parts.push({ end: range.end, start: from });
  return parts;
};

const withRate = (cut: SourceCut, part: Range): SourceCut => ({
  ...cut,
  sourceEnd: part.end,
  sourceStart: part.start,
});

/** The recorded cuts, cut down to what the edit still leaves out. */
export const stillRemoved = (
  segments: readonly RecordingTimelineSegment[],
  cuts: readonly SourceCut[],
): SourceCut[] =>
  cuts.flatMap((cut) =>
    removedParts(segments, {
      end: cut.sourceEnd,
      start: cut.sourceStart,
    }).map((part) => withRate(cut, part)),
  );

export type CutResult = {
  /** Newly split segments, keyed by their parent, for the keyboard edits to
   * follow. */
  descendantIds: Map<number, number[]>;
  nextSegmentId: number;
  segments: RecordingTimelineSegment[];
  /** What was taken out, in source order, at the rate each played at. */
  taken: SourceCut[];
};

/**
 * Splits the segments around the source ranges, keeping what the ranges do
 * not cover and reporting what they take. `ranges` are shares, sorted by
 * start and not overlapping. A cut never takes the whole recording: one that
 * would is left out.
 */
export const cutSegments = (
  edit: RecordingTimelineEdit,
  ranges: readonly Range[],
): CutResult => {
  let nextSegmentId = edit.nextSegmentId;
  const descendantIds = new Map<number, number[]>();
  const taken: SourceCut[] = [];
  const segments = edit.segments.flatMap((segment) => {
    const inside = ranges.filter(
      (range) =>
        range.end > segment.sourceStart && range.start < segment.sourceEnd,
    );
    if (inside.length === 0) return [segment];
    const pieces: RecordingTimelineSegment[] = [];
    let from = segment.sourceStart;
    const keep = (end: number) => {
      if (end - from <= CUT_EPSILON) return;
      pieces.push({
        ...segment,
        id: pieces.length === 0 ? segment.id : nextSegmentId++,
        sourceEnd: end,
        sourceStart: from,
      });
    };
    for (const range of inside) {
      const start = Math.max(range.start, segment.sourceStart);
      const end = Math.min(range.end, segment.sourceEnd);
      keep(start);
      taken.push({
        playbackRate: segment.playbackRate ?? 1,
        sourceEnd: end,
        sourceStart: start,
      });
      from = Math.max(from, end);
    }
    keep(segment.sourceEnd);
    // The first piece kept carries the segment on; any piece after it is
    // new and takes the segment's shortcut edits with it.
    const descendants = pieces.slice(1).map((piece) => piece.id);
    if (descendants.length > 0) descendantIds.set(segment.id, descendants);
    return pieces;
  });
  return { descendantIds, nextSegmentId, segments, taken };
};

/** Milliseconds of a recording `sourceDurationMs` long, as shares, clamped,
 * empty ones dropped and the rest sorted. */
export const sharesOf = (
  ranges: readonly { endMs: number; startMs: number }[],
  sourceDurationMs: number,
): Range[] =>
  ranges
    .map((range) => ({
      end: Math.min(1, range.endMs / sourceDurationMs),
      start: Math.max(0, range.startMs / sourceDurationMs),
    }))
    .filter((range) => range.end - range.start > CUT_EPSILON)
    .sort((a, b) => a.start - b.start);

/**
 * Lays `added` segments back into the edit's segments and folds each one
 * into a neighbour it meets at the same rate, the way restoring a cut joins
 * the two halves it split apart. Each goes after the segment that ends where
 * it starts in the source, wherever that plays, or before the segment
 * earliest in the source when none does. A segment carrying a shortcut edit
 * never folds, so its edits keep their own stretch.
 */
export const mergeAddedSegments = (
  edit: RecordingTimelineEdit,
  added: readonly RecordingTimelineSegment[],
): RecordingTimelineSegment[] => {
  const restored = new Set(added.map((segment) => segment.id));
  const inserted = [...edit.segments];
  for (const segment of [...added].sort(
    (a, b) => a.sourceStart - b.sourceStart,
  )) {
    const predecessor = sourcePredecessorIndex(inserted, segment.sourceStart);
    const at =
      predecessor >= 0 ? predecessor + 1 : earliestSourceIndex(inserted);
    inserted.splice(Math.max(0, at), 0, segment);
  }
  const joined = new Set<number>();
  const segments: RecordingTimelineSegment[] = [];
  for (const segment of inserted) {
    const previous =
      segments.length > 0 ? segments[segments.length - 1] : undefined;
    const joins =
      previous &&
      (restored.has(previous.id) ||
        joined.has(previous.id) ||
        restored.has(segment.id)) &&
      Math.abs(segment.sourceStart - previous.sourceEnd) <= CUT_EPSILON &&
      (previous.playbackRate ?? 1) === (segment.playbackRate ?? 1) &&
      !hasKeyboardSegmentEdits(edit, previous.id) &&
      !hasKeyboardSegmentEdits(edit, segment.id);
    if (!joins) {
      segments.push(segment);
      continue;
    }
    // The joined segment keeps an id the edit already knew where it can.
    const kept = restored.has(previous.id) ? segment : previous;
    segments[segments.length - 1] = {
      ...kept,
      sourceEnd: segment.sourceEnd,
      sourceStart: previous.sourceStart,
    };
    joined.add(kept.id);
  }
  return segments;
};
