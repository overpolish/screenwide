// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cutKeyboardTimeline } from "../keyboard/recording-keyboard-timeline-cut";

import {
  layoutRecordingTimelineSegments,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceEndToOutput,
  recordingTimelineSourceToOutput,
} from "./recording-timeline-edit";

import type { RecordingTimelineEdit } from "../recording-timeline-types";

/** A cut closer than this to a segment's own edge, in source milliseconds,
 * is the edge: clip and word times are whole milliseconds, and a cut
 * between segments need not be. */
const EDGE_MS = 1;
/** How far apart, in output milliseconds, two places on the timeline still
 * count as one. Larger than `EDGE_MS` at the slowest playback rate. */
const SAME_PLACE_MS = 5;

/** `edit` with a cut at `share` of the recording, unless that falls within
 * `margin` of the edge of the segment holding it or in footage none plays. */
const cutNear = (
  edit: RecordingTimelineEdit,
  share: number,
  margin: number,
): RecordingTimelineEdit =>
  edit.segments.some(
    (segment) =>
      share > segment.sourceStart + margin &&
      share < segment.sourceEnd - margin,
  )
    ? cutKeyboardTimeline(edit, share)
    : edit;

/**
 * Moves the footage the timeline plays from the moment `startMs` into the
 * recording to the moment `endMs`, screen, camera and audio together, to
 * play just before the moment `beforeMs`, or at the end of the timeline for
 * null. Every segment is cut where the footage starts and ends and where it
 * lands, and the pieces it covers move in the order they played; the rest
 * close up behind them. A moment the timeline cut away lands where it would
 * be restored, after the footage before it in the recording. The footage
 * may run across a jump of a reordered timeline: what plays between its
 * ends moves. A move onto the footage itself or onto either of its ends
 * returns the edit itself, which keeps it out of undo history.
 */
export function moveRecordingTimelineFootage(
  edit: RecordingTimelineEdit,
  {
    beforeMs,
    endMs,
    startMs,
  }: { beforeMs: number | null; endMs: number; startMs: number },
  sourceDurationMs: number,
): RecordingTimelineEdit {
  if (sourceDurationMs <= 0) return edit;
  const share = (ms: number) => ms / sourceDurationMs;
  const margin = EDGE_MS / sourceDurationMs;
  let cut = cutNear(edit, share(startMs), margin);
  cut = cutNear(cut, share(endMs), margin);
  if (beforeMs !== null) cut = cutNear(cut, share(beforeMs), margin);
  const from = recordingTimelineSourceToOutput(cut, share(startMs));
  const to = recordingTimelineSourceEndToOutput(cut, share(endMs));
  const at =
    beforeMs === null
      ? 1
      : recordingTimelineSourceToOutput(cut, share(beforeMs));
  const tolerance =
    SAME_PLACE_MS /
    Math.max(1, recordingTimelineRetainedDuration(cut) * sourceDurationMs);
  if (to - from <= tolerance || (at > from - tolerance && at < to + tolerance))
    return edit;
  const laidOut = layoutRecordingTimelineSegments(cut);
  const isMoved = (segment: (typeof laidOut)[number]) =>
    segment.outputStart > from - tolerance &&
    segment.outputEnd < to + tolerance;
  const moved = laidOut.filter(isMoved);
  if (moved.length === 0) return edit;
  const staying = laidOut.filter((segment) => !isMoved(segment));
  const index = staying.filter(
    (segment) => segment.outputEnd < at + tolerance,
  ).length;
  const segments = [
    ...staying.slice(0, index),
    ...moved,
    ...staying.slice(index),
  ].map(({ outputEnd: _end, outputStart: _start, ...segment }) => segment);
  return { ...cut, segments };
}
