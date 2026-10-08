// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { inheritRecordingKeyboardSegmentEdits } from "../keyboard/recording-keyboard-timeline-edit";

import { hasKeyboardSegmentEdits } from "./recording-timeline-cuts";
import {
  RecordingTimelineEdit,
  RecordingTimelineSegment,
} from "./recording-timeline-edit";

import type { RecordingSilenceCut } from "../../../../bindings/RecordingSilenceCut";
import type { SilenceCut } from "../../../../bindings/SilenceCut";

export type { SilenceCut };

const EPSILON = 1e-9;

type Range = { end: number; start: number };

/** What of `range` no segment keeps, in source order. */
const removedParts = (
  segments: readonly RecordingTimelineSegment[],
  range: Range,
): Range[] => {
  const parts: Range[] = [];
  let from = range.start;
  for (const segment of segments) {
    if (segment.sourceEnd <= from) continue;
    if (segment.sourceStart >= range.end) break;
    if (segment.sourceStart - from > EPSILON)
      parts.push({ end: segment.sourceStart, start: from });
    from = Math.max(from, segment.sourceEnd);
  }
  if (range.end - from > EPSILON) parts.push({ end: range.end, start: from });
  return parts;
};

const withRate = (
  cut: RecordingSilenceCut,
  part: Range,
): RecordingSilenceCut => ({
  ...cut,
  sourceEnd: part.end,
  sourceStart: part.start,
});

/** The recorded silences, cut down to what is still cut out. */
const stillRemoved = (edit: RecordingTimelineEdit) =>
  (edit.silenceCuts ?? []).flatMap((cut) =>
    removedParts(edit.segments, {
      end: cut.sourceEnd,
      start: cut.sourceStart,
    }).map((part) => withRate(cut, part)),
  );

/**
 * How many of the pauses Remove silences cut are still cut, and how much
 * shorter they make the recording, in milliseconds of the recording at the
 * rate each played at.
 */
export function recordingTimelineSilenceSummary(
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
) {
  const removed = stillRemoved(edit);
  return {
    count: (edit.silenceCuts ?? []).filter(
      (cut) =>
        removedParts(edit.segments, {
          end: cut.sourceEnd,
          start: cut.sourceStart,
        }).length > 0,
    ).length,
    durationMs: removed.reduce(
      (total, cut) =>
        total +
        ((cut.sourceEnd - cut.sourceStart) * sourceDurationMs) /
          cut.playbackRate,
      0,
    ),
  };
}

/**
 * Cuts `cuts`, in milliseconds of a recording `sourceDurationMs` long, out of
 * whatever of them the edit still keeps, and records what it took for Restore
 * all. A cut never takes the whole recording; one that would is left out.
 */
export function removeRecordingTimelineSilences(
  edit: RecordingTimelineEdit,
  cuts: readonly SilenceCut[],
  sourceDurationMs: number,
): RecordingTimelineEdit {
  if (sourceDurationMs <= 0) return edit;
  const ranges = cuts
    .map((cut) => ({
      end: Math.min(1, cut.endMs / sourceDurationMs),
      start: Math.max(0, cut.startMs / sourceDurationMs),
    }))
    .filter((range) => range.end - range.start > EPSILON)
    .sort((a, b) => a.start - b.start);
  let nextSegmentId = edit.nextSegmentId;
  const descendantIds = new Map<number, number[]>();
  const taken: RecordingSilenceCut[] = [];
  const segments = edit.segments.flatMap((segment) => {
    const inside = ranges.filter(
      (range) =>
        range.end > segment.sourceStart && range.start < segment.sourceEnd,
    );
    if (inside.length === 0) return [segment];
    const pieces: RecordingTimelineSegment[] = [];
    let from = segment.sourceStart;
    const keep = (end: number) => {
      if (end - from <= EPSILON) return;
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
  if (taken.length === 0 || segments.length === 0) return edit;
  return inheritRecordingKeyboardSegmentEdits(
    edit,
    {
      ...edit,
      nextSegmentId,
      segments,
      silenceCuts: [...stillRemoved(edit), ...taken].sort(
        (a, b) => a.sourceStart - b.sourceStart,
      ),
    },
    descendantIds,
  );
}

/**
 * Brings back every part of the pauses Remove silences cut that is still cut,
 * each at the rate it played at. A restored part that meets a neighbour at
 * the same rate becomes one with it, unless either carries a shortcut edit.
 */
export function restoreRecordingTimelineSilences(
  edit: RecordingTimelineEdit,
): RecordingTimelineEdit {
  const removed = stillRemoved(edit);
  if (removed.length === 0) {
    if (!edit.silenceCuts?.length) return edit;
    const { silenceCuts: _, ...rest } = edit;
    return rest;
  }
  let nextSegmentId = edit.nextSegmentId;
  const restored = new Set<number>();
  const inserted = [
    ...edit.segments,
    ...removed.map((cut): RecordingTimelineSegment => {
      const id = nextSegmentId++;
      restored.add(id);
      return {
        id,
        sourceEnd: cut.sourceEnd,
        sourceStart: cut.sourceStart,
        ...(cut.playbackRate === 1 ? {} : { playbackRate: cut.playbackRate }),
      };
    }),
  ].sort((a, b) => a.sourceStart - b.sourceStart);
  // A segment a restored part has joined carries the join on: the cut split
  // it from its other half, which meets it once the part is back.
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
      Math.abs(segment.sourceStart - previous.sourceEnd) <= EPSILON &&
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
  const { silenceCuts: _, ...rest } = edit;
  return { ...rest, nextSegmentId, segments };
}
