// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  isGapSegment,
  type RecordingTimelineEdit,
  type RecordingTimelineSegment,
} from "../recording-timeline-types";

import { cutAt, joinedAt } from "./recording-strip-source";

// Gaps: time inserted into the timeline with nothing recorded behind it,
// for a title card, a header or a pause between parts. A gap is a segment
// past the recording's end (a source of one or more), each in a slot of
// that empty source of its own an hour long, so no two ever meet and every
// moment of the timeline keeps a source moment of its own; the twin of
// `RecordingTimelineSegment::is_gap` in the `timeline_edit` crate.
// Annotations and scenes are carried onto the timeline a gap changes by the
// footage they sit on, as for every edit, and no scene plays over a gap
// (`recording-scene-gaps.ts`), which is drawn as a blank scene of its own,
// on a background of its own where it has one; the songs and sound effects,
// laid under the timeline, move with what comes after the gap themselves.

/** How long each gap's slot of the empty source is, in ms. */
const GAP_SLOT_MS = 3_600_000;
/** How long a gap may be at most, and at least, in ms. */
export const MAX_GAP_MS = 600_000;
const MIN_GAP_MS = 100;
/** How long a gap is inserted, in ms. */
export const DEFAULT_GAP_MS = 3_000;

/** Within this of a moment, in ms, is that moment. */
const SAME_MS = 0.5;

const GAP_BEAT_PREFIX = "gap-";

/** The id the strip gives the gap segment `segmentId` among its beats. */
export const gapBeatId = (segmentId: number) =>
  `${GAP_BEAT_PREFIX}${String(segmentId)}`;

/** The gap segment a beat on the strip is, by its id, or null for a beat
 * of the recording. */
export const gapSegmentIdOf = (beatId: string) =>
  beatId.startsWith(GAP_BEAT_PREFIX)
    ? Number(beatId.slice(GAP_BEAT_PREFIX.length))
    : null;

/** How long `segment` plays, in ms, on a recording `durationMs` long. */
export const playedMs = (
  segment: RecordingTimelineSegment,
  durationMs: number,
) =>
  ((segment.sourceEnd - segment.sourceStart) * durationMs) /
  (segment.playbackRate ?? 1);

/** Where each of `segments` starts on the timeline, in ms. */
export const startsMs = (
  segments: readonly RecordingTimelineSegment[],
  durationMs: number,
) => {
  let at = 0;
  return segments.map((segment) => {
    const start = at;
    at += playedMs(segment, durationMs);
    return start;
  });
};

const clampLength = (lengthMs: number) =>
  Math.min(MAX_GAP_MS, Math.max(MIN_GAP_MS, Math.round(lengthMs)));

/** `edit`'s songs and sound effects that start at `fromMs` or later moved
 * by `byMs`, in whole ms as clips are timed; those already playing there
 * keep playing through. */
const shiftedAudio = (
  edit: RecordingTimelineEdit,
  fromMs: number,
  byMs: number,
): Pick<RecordingTimelineEdit, "audioClips"> => {
  const by = Math.round(byMs);
  return edit.audioClips && by !== 0
    ? {
        audioClips: edit.audioClips.map((clip) =>
          clip.startMs >= fromMs - SAME_MS
            ? {
                ...clip,
                endMs: Math.max(0, clip.endMs + by),
                startMs: Math.max(0, clip.startMs + by),
              }
            : clip,
        ),
      }
    : {};
};

/** The share of the recording where a gap's slot `slot` begins. */
const slotStart = (slot: number, durationMs: number) =>
  1 + (slot * GAP_SLOT_MS) / durationMs;

/** The slot the gap `segment` lies in. */
const slotOf = (segment: RecordingTimelineSegment, durationMs: number) =>
  Math.floor(((segment.sourceStart - 1) * durationMs) / GAP_SLOT_MS);

/** The first slot of the empty source no gap of `edit` holds. Slot 0 is
 * never used, so no gap starts where the recording ends. */
function freeSlot(edit: RecordingTimelineEdit, durationMs: number) {
  const taken = new Set(
    edit.segments
      .filter(isGapSegment)
      .map((segment) => slotOf(segment, durationMs)),
  );
  let slot = 1;
  while (taken.has(slot)) slot += 1;
  return slot;
}

/**
 * `edit` with a gap `lengthMs` long inserted `atMs` into the timeline, on a
 * recording `durationMs` long: the recording is split there and what comes
 * after moves later. Inside a gap, that gap grows instead.
 */
export function insertRecordingTimelineGap(
  edit: RecordingTimelineEdit,
  {
    atMs,
    durationMs,
    lengthMs,
  }: {
    atMs: number;
    durationMs: number;
    lengthMs: number;
  },
): RecordingTimelineEdit {
  if (durationMs <= 0) return edit;
  const length = clampLength(lengthMs);
  const starts = startsMs(edit.segments, durationMs);
  const holding = edit.segments.findIndex(
    (segment, index) =>
      atMs > starts[index] + SAME_MS &&
      atMs < starts[index] + playedMs(segment, durationMs) - SAME_MS,
  );
  const held = edit.segments[holding] as RecordingTimelineSegment | undefined;
  if (held && isGapSegment(held))
    return resizeRecordingTimelineGap(edit, {
      durationMs,
      id: held.id,
      lengthMs: playedMs(held, durationMs) + length,
    });
  // Split where the gap goes, unless that is already a segment's edge.
  const split = held
    ? cutAt(
        edit,
        held.sourceStart +
          ((atMs - starts[holding]) * (held.playbackRate ?? 1)) / durationMs,
      )
    : edit;
  const splitStarts = startsMs(split.segments, durationMs);
  const index = splitStarts.findIndex((start) => start >= atMs - SAME_MS);
  const at = index < 0 ? split.segments.length : index;
  // Placed a longest gap into its slot, so its start can be drawn out too.
  const start =
    slotStart(freeSlot(split, durationMs), durationMs) +
    MAX_GAP_MS / durationMs;
  const gap: RecordingTimelineSegment = {
    id: split.nextSegmentId,
    sourceEnd: start + length / durationMs,
    sourceStart: start,
  };
  return {
    ...split,
    ...shiftedAudio(edit, atMs, length),
    nextSegmentId: split.nextSegmentId + 1,
    segments: [
      ...split.segments.slice(0, at),
      gap,
      ...split.segments.slice(at),
    ],
  };
}

/** `edit` with the gap `id` made `lengthMs` long, what comes after it moved
 * with its end. */
export function resizeRecordingTimelineGap(
  edit: RecordingTimelineEdit,
  {
    durationMs,
    id,
    lengthMs,
  }: {
    durationMs: number;
    id: number;
    lengthMs: number;
  },
): RecordingTimelineEdit {
  const index = edit.segments.findIndex(
    (segment) => segment.id === id && isGapSegment(segment),
  );
  if (index < 0 || durationMs <= 0) return edit;
  const gap = edit.segments[index];
  const before = playedMs(gap, durationMs);
  const length = clampLength(lengthMs);
  if (Math.abs(length - before) < SAME_MS) return edit;
  const endMs = startsMs(edit.segments, durationMs)[index] + before;
  const segments = [...edit.segments];
  segments[index] = {
    ...gap,
    sourceEnd: gap.sourceStart + length / durationMs,
  };
  return { ...edit, ...shiftedAudio(edit, endMs, length - before), segments };
}

/** `edit` without the gap `id`, what came after it moved back, and the
 * recording made one again where the gap had split it. A timeline keeps
 * some of the recording, so its last stretch is never taken for a gap's
 * sake. */
export function deleteRecordingTimelineGap(
  edit: RecordingTimelineEdit,
  { durationMs, id }: { durationMs: number; id: number },
): RecordingTimelineEdit {
  const index = edit.segments.findIndex(
    (segment) => segment.id === id && isGapSegment(segment),
  );
  if (index < 0 || durationMs <= 0) return edit;
  const gap = edit.segments[index];
  const length = playedMs(gap, durationMs);
  const startMs = startsMs(edit.segments, durationMs)[index];
  const segments = edit.segments.filter((segment) => segment.id !== id);
  const after = segments[index] as RecordingTimelineSegment | undefined;
  const removed: RecordingTimelineEdit = {
    ...edit,
    ...shiftedAudio(edit, startMs + length, -length),
    segments,
  };
  return after && !isGapSegment(after)
    ? joinedAt(removed, [after.sourceStart])
    : removed;
}

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

/**
 * `edit` with an end of the gap `id` dragged to `atMs` of its own empty
 * source, as a beat's end is trimmed: in, the gap shortens, out, it grows,
 * within `gapTrimReach` and no shorter than a gap may be. Each end is a
 * moment of the gap's own stretch, so the same drag gives the same gap
 * however often it is told; what comes after the gap moves with its end.
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
  const index = edit.segments.findIndex(
    (segment) => segment.id === id && isGapSegment(segment),
  );
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
