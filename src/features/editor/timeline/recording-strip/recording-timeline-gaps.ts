// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  isGapSegment,
  type RecordingTimelineEdit,
} from "../recording-timeline-types";

import { playedMs, startsMs } from "./recording-strip-gaps";

import type { Background } from "../../../../bindings/Background";
import type { GapScene } from "../../../../bindings/GapScene";

// The gaps as the rest of the editor reads them (`recording-strip-gaps.ts`
// makes and changes them): where each plays and what it wears, which the
// picture draws as a blank scene, and whether a moment lies in one.

/** `edit` with the gap `id` wearing `background`, or the recording's own
 * again for null. */
export function setRecordingTimelineGapBackground(
  edit: RecordingTimelineEdit,
  { background, id }: { background: Background | null; id: number },
): RecordingTimelineEdit {
  if (
    !edit.segments.some((segment) => segment.id === id && isGapSegment(segment))
  )
    return edit;
  return {
    ...edit,
    segments: edit.segments.map((segment) => {
      if (segment.id !== id) return segment;
      const { background: _worn, ...bare } = segment;
      return background ? { ...bare, background } : bare;
    }),
  };
}

/** The gaps `edit` plays on a recording `durationMs` long, in order, in ms
 * on the timeline, each with its own background where it has one: the
 * twin of `TimelinePlan::gap_scenes`, which the picture draws as blank
 * scenes. */
export function recordingTimelineGaps(
  edit: RecordingTimelineEdit,
  durationMs: number,
): GapScene[] {
  if (durationMs <= 0) return [];
  const starts = startsMs(edit.segments, durationMs);
  return edit.segments.flatMap((segment, index) =>
    isGapSegment(segment)
      ? [
          {
            endMs: Math.round(starts[index] + playedMs(segment, durationMs)),
            startMs: Math.round(starts[index]),
            ...(segment.background ? { background: segment.background } : {}),
          },
        ]
      : [],
  );
}

/** Whether `sourceMs` into a recording `durationMs` long lies in one of
 * `edit`'s gaps, past the recording's end. */
export const isInRecordingTimelineGap = (
  edit: RecordingTimelineEdit,
  durationMs: number,
  sourceMs: number,
) =>
  durationMs > 0 &&
  edit.segments.some(
    (segment) =>
      isGapSegment(segment) &&
      sourceMs >= segment.sourceStart * durationMs &&
      sourceMs < segment.sourceEnd * durationMs,
  );
