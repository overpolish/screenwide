// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { recordingTimelineCuts } from "../editing/recording-timeline-cuts";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { cutKeyboardTimeline } from "../keyboard/recording-keyboard-timeline-cut";

import { isJoinable } from "./recording-strip-time";

// The timeline edit changed by stretches of the recording, as the strip
// names them, wherever they play: taken out, brought back, played at a
// speed, and made one again where the strip's own cuts leave segments that
// meet as they were recorded. Positions are shares of the recording. Every
// change that changes nothing returns the edit itself, which keeps it out
// of undo history.

/** Closer than this to a segment's edge, as a share of the recording, is
 * the edge. */
const EDGE = 1e-6;

/** `edit` cut at `at`, unless that is at a segment's edge or in a stretch
 * no segment plays. */
export const cutAt = (edit: RecordingTimelineEdit, at: number) =>
  edit.segments.some(
    (segment) =>
      at > segment.sourceStart + EDGE && at < segment.sourceEnd - EDGE,
  )
    ? cutKeyboardTimeline(edit, at)
    : edit;

const isInside = (
  segment: { sourceEnd: number; sourceStart: number },
  start: number,
  end: number,
) => segment.sourceStart >= start - EDGE && segment.sourceEnd <= end + EDGE;

/** `edit` with the segments that meet at each of `moments`, as they were
 * recorded, made one where nothing holds them apart. */
export function joinedAt(
  edit: RecordingTimelineEdit,
  moments: number[],
): RecordingTimelineEdit {
  let segments = edit.segments;
  for (const at of moments) {
    const index = segments.findIndex(
      (segment, each) =>
        each > 0 &&
        Math.abs(segment.sourceStart - at) < EDGE &&
        isJoinable(edit, segments[each - 1], segment),
    );
    if (index < 0) continue;
    segments = [
      ...segments.slice(0, index - 1),
      { ...segments[index - 1], sourceEnd: segments[index].sourceEnd },
      ...segments.slice(index + 1),
    ];
  }
  return segments === edit.segments ? edit : { ...edit, segments };
}

/** `edit` without the recording from `start` to `end`, wherever it plays;
 * never without the whole of it. */
export function removed(
  edit: RecordingTimelineEdit,
  start: number,
  end: number,
): RecordingTimelineEdit {
  if (end - start <= EDGE) return edit;
  const cut = cutAt(cutAt(edit, start), end);
  const segments = cut.segments.filter(
    (segment) => !isInside(segment, start, end),
  );
  if (segments.length === 0 || segments.length === cut.segments.length)
    return edit;
  return joinedAt({ ...cut, segments }, [start]);
}

/** `edit` playing again what it left out from `start` to `end`: each stretch
 * left out there grows the segment it meets back over it. */
export function restored(
  edit: RecordingTimelineEdit,
  start: number,
  end: number,
): RecordingTimelineEdit {
  let next = edit;
  const meets: number[] = [];
  for (const cut of recordingTimelineCuts(edit)) {
    const from = Math.max(cut.sourceStart, start);
    const to = Math.min(cut.sourceEnd, end);
    if (to - from <= EDGE) continue;
    const segments = [...next.segments];
    const grow = (
      id: number | null,
      change: { sourceEnd: number } | { sourceStart: number },
    ) => {
      const index = segments.findIndex((segment) => segment.id === id);
      if (index < 0) return false;
      segments[index] = { ...segments[index], ...change };
      return true;
    };
    // Met at its end, a stretch grows the segment that follows it back to
    // `from`; met at its start, or at the recording's end, the one before
    // it on to `to`.
    const hasGrown =
      (to >= cut.sourceEnd - EDGE &&
        grow(cut.followingSegmentId, { sourceStart: from })) ||
      (from <= cut.sourceStart + EDGE &&
        grow(cut.segmentId, { sourceEnd: to }));
    if (!hasGrown) continue;
    next = { ...next, segments };
    meets.push(from, to);
  }
  return next === edit ? edit : joinedAt(next, meets);
}

/** `edit` playing the recording from `start` to `end` at `rate` times as
 * fast as recorded. */
export function atRate(
  edit: RecordingTimelineEdit,
  { end, rate, start }: { end: number; rate: number; start: number },
): RecordingTimelineEdit {
  if (end - start <= EDGE || !(rate >= 0.25 && rate <= 4)) return edit;
  const cut = cutAt(cutAt(edit, start), end);
  const isChanged = cut.segments.some(
    (segment) =>
      isInside(segment, start, end) && (segment.playbackRate ?? 1) !== rate,
  );
  if (!isChanged) return edit;
  const segments = cut.segments.map((segment) =>
    isInside(segment, start, end)
      ? { ...segment, playbackRate: rate }
      : segment,
  );
  const inside = segments.flatMap((segment) =>
    isInside(segment, start, end) ? [segment.sourceStart] : [],
  );
  return joinedAt({ ...cut, segments }, [start, end, ...inside]);
}
