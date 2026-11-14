// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  isGapSegment,
  type RecordingTimelineEdit,
  type RecordingTimelineSegment,
} from "../recording-timeline-types";

import {
  deleteRecordingTimelineGap,
  MAX_GAP_MS,
  MIN_GAP_MS,
  placedGap,
  playedMs,
  SAME_MS,
  shiftedAudio,
  slotOf,
  slotStart,
  startsMs,
} from "./recording-strip-gaps";

// A gap's edges and its place, drawn on the strip. Either end dragged in or
// out shortens or lengthens it, nothing of the recording cut, what follows
// moving with it; the strip keeps the start under the pointer by moving its
// view, so what came before is pushed back as the gap grows (user asked,
// 2026-11-14). The gap dragged whole slides along the timeline, the place it
// splits the recording moving with it (user asked, 2026-11-14). Each end is
// told as a moment of the gap's own empty stretch and its place as a moment
// of the timeline, so the same drag gives the same gap however often it is
// told.

/** How far the gap `segment` on a recording `durationMs` long may be drawn
 * out, in recorded seconds, as its beat is: its start back as far as its
 * slot begins, and either end until the gap is as long as a gap may be. */
export function gapTrimReach(
  segment: RecordingTimelineSegment,
  durationMs: number,
): { end: number; start: number } {
  const recorded = durationMs / 1000;
  const most = MAX_GAP_MS / 1000;
  return {
    end: segment.sourceStart * recorded + most,
    start: Math.max(
      slotStart(slotOf(segment, durationMs), durationMs) * recorded,
      segment.sourceEnd * recorded - most,
    ),
  };
}

const gapIndex = (edit: RecordingTimelineEdit, id: number) =>
  edit.segments.findIndex(
    (segment) => segment.id === id && isGapSegment(segment),
  );

/**
 * `edit` with an end of the gap `id` dragged to `atMs` of its own empty
 * source, on a recording `durationMs` long, within `gapTrimReach` and no
 * shorter than a gap may be: the gap shortened or lengthened, what comes
 * after moving with it.
 */
export function trimRecordingTimelineGap(
  edit: RecordingTimelineEdit,
  {
    atMs,
    durationMs,
    edge,
    id,
  }: { atMs: number; durationMs: number; edge: "end" | "start"; id: number },
): RecordingTimelineEdit {
  const index = gapIndex(edit, id);
  if (index < 0 || durationMs <= 0) return edit;
  const gap = edit.segments[index];
  const reach = gapTrimReach(gap, durationMs);
  const at =
    Math.min(Math.max(atMs / 1000, reach.start), reach.end) /
    (durationMs / 1000);
  const least = MIN_GAP_MS / durationMs;
  const trimmed =
    edge === "start"
      ? { ...gap, sourceStart: Math.min(at, gap.sourceEnd - least) }
      : { ...gap, sourceEnd: Math.max(at, gap.sourceStart + least) };
  const before = playedMs(gap, durationMs);
  const after = playedMs(trimmed, durationMs);
  if (Math.abs(after - before) < SAME_MS) return edit;
  const endMs = startsMs(edit.segments, durationMs)[index] + before;
  const segments = [...edit.segments];
  segments[index] = trimmed;
  return { ...edit, ...shiftedAudio(edit, endMs, after - before), segments };
}

/**
 * `edit` with the gap `id` slid to start `atMs` into the timeline, on a
 * recording `durationMs` long: taken out where it was, the recording made
 * one again there, and played from `atMs` of what is left, the recording
 * split there. It keeps its length and background, and what is drawn over
 * it slides with it; the songs and sound effects stay with the recording
 * either side of both places.
 */
export function slideRecordingTimelineGap(
  edit: RecordingTimelineEdit,
  { atMs, durationMs, id }: { atMs: number; durationMs: number; id: number },
): RecordingTimelineEdit {
  const index = gapIndex(edit, id);
  if (index < 0 || durationMs <= 0) return edit;
  const gap = edit.segments[index];
  const taken = deleteRecordingTimelineGap(edit, { durationMs, id });
  const rest = taken.segments.reduce(
    (sum, segment) => sum + playedMs(segment, durationMs),
    0,
  );
  const at = Math.min(Math.max(atMs, 0), rest);
  const was = startsMs(edit.segments, durationMs)[index];
  if (Math.abs(at - was) < SAME_MS) return edit;
  return placedGap(taken, { atMs: at, durationMs, gap });
}
