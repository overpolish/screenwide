// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Annotation and scene clips are timed on the edited video, in milliseconds.
// These carry a moment between that timeline and the recording, and carry
// clips onto a timeline whose segments changed; the twin of
// `carry_clips_onto_timeline` in the `timeline_edit` crate.

import {
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
} from "./recording-timeline-edit";
import { recordingTimelineOutputToSourceStart } from "./recording-timeline-source-order";

import type {
  RecordingTimelineEdit,
  RecordingTimelineSegment,
} from "../recording-timeline-types";

type Timed = { endMs: number; startMs: number };

/** How long the edited video plays, in milliseconds. */
export const recordingTimelineLengthMs = (
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
) => recordingTimelineRetainedDuration(edit) * Math.max(0, sourceDurationMs);

/** Where the moment `sourceMs` into the recording plays on the timeline, in
 * milliseconds: as the start of whatever begins there. */
export const recordingTimelineMsAt = (
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
  sourceMs: number,
) =>
  sourceDurationMs > 0
    ? recordingTimelineSourceToOutput(edit, sourceMs / sourceDurationMs) *
      recordingTimelineLengthMs(edit, sourceDurationMs)
    : 0;

/** The moment of the recording that starts playing `timelineMs` into the
 * edited video, in milliseconds. */
export const recordingMsAt = (
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
  timelineMs: number,
) => {
  const lengthMs = recordingTimelineLengthMs(edit, sourceDurationMs);
  return lengthMs > 0
    ? recordingTimelineOutputToSourceStart(
        edit.segments,
        timelineMs / lengthMs,
      ) * sourceDurationMs
    : 0;
};

/** The stretch of the recording a clip on the timeline plays, in
 * milliseconds: from what starts playing at its start to what stops at its
 * end. A clip across a jump on a reordered timeline plays more than one
 * stretch; this spans from the first to the last. */
export const recordingSpanMs = (
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
  { endMs, startMs }: Timed,
): [number, number] => {
  const lengthMs = recordingTimelineLengthMs(edit, sourceDurationMs);
  if (lengthMs <= 0) return [startMs, endMs];
  return [
    recordingMsAt(edit, sourceDurationMs, startMs),
    recordingTimelineOutputToSource(edit, endMs / lengthMs) * sourceDurationMs,
  ];
};

/** A segment's place on the timeline, in milliseconds: the recording
 * `[sourceStart, sourceEnd)` it plays, from `timelineStart` on, at `rate`. */
type Placed = {
  rate: number;
  sourceEnd: number;
  sourceStart: number;
  timelineStart: number;
};

const placed = (
  segments: readonly RecordingTimelineSegment[],
  sourceDurationMs: number,
): Placed[] => {
  let timeline = 0;
  return segments.map((segment) => {
    const rate = segment.playbackRate ?? 1;
    const sourceStart = segment.sourceStart * sourceDurationMs;
    const sourceEnd = segment.sourceEnd * sourceDurationMs;
    const place = { rate, sourceEnd, sourceStart, timelineStart: timeline };
    timeline += (sourceEnd - sourceStart) / rate;
    return place;
  });
};

const timelineEnd = (segment: Placed) =>
  segment.timelineStart +
  (segment.sourceEnd - segment.sourceStart) / segment.rate;

/** The stretches of the recording the timeline `[startMs, endMs)` plays. */
const recordingPieces = (segments: Placed[], { endMs, startMs }: Timed) =>
  segments.flatMap((segment) => {
    const from = Math.max(startMs, segment.timelineStart);
    const to = Math.min(endMs, timelineEnd(segment));
    return from < to
      ? [
          [
            segment.sourceStart + (from - segment.timelineStart) * segment.rate,
            segment.sourceStart + (to - segment.timelineStart) * segment.rate,
          ] as const,
        ]
      : [];
  });

/** The stretches of the timeline the recording `[start, end)` plays on. */
const timelinePieces = (
  segments: Placed[],
  [start, end]: readonly [number, number],
) =>
  segments.flatMap((segment) => {
    const from = Math.max(start, segment.sourceStart);
    const to = Math.min(end, segment.sourceEnd);
    return from < to
      ? [
          [
            segment.timelineStart + (from - segment.sourceStart) / segment.rate,
            segment.timelineStart + (to - segment.sourceStart) / segment.rate,
          ] as [number, number],
        ]
      : [];
  });

/** Less than this between two stretches, in milliseconds, is rounding: clip
 * times are whole milliseconds and a cut between segments need not be. */
const JOIN_MS = 0.5;

/** An edit's change of segments, from `previous` to `next`, on a recording
 * `sourceDurationMs` long. */
export type RecordingTimelineChange = {
  next: readonly RecordingTimelineSegment[];
  previous: readonly RecordingTimelineSegment[];
  sourceDurationMs: number;
};

/**
 * Where `clip`, timed on the timeline `previous` lays out, plays on the one
 * `next` lays out, in whole milliseconds: one stretch for each place its
 * footage still plays, in timeline order, with stretches that meet joined.
 * Footage `next` no longer plays is left behind, so a clip it cuts away
 * entirely has no stretches.
 */
function retimedStretches(
  clip: Timed,
  { next, previous, sourceDurationMs }: RecordingTimelineChange,
): Timed[] {
  const from = placed(previous, sourceDurationMs);
  const to = placed(next, sourceDurationMs);
  const stretches = recordingPieces(from, clip)
    .flatMap((piece) => timelinePieces(to, piece))
    .sort((a, b) => a[0] - b[0]);
  const joined: [number, number][] = [];
  for (const stretch of stretches) {
    const last = joined[joined.length - 1] as [number, number] | undefined;
    if (last && stretch[0] <= last[1] + JOIN_MS) {
      last[1] = Math.max(last[1], stretch[1]);
    } else {
      joined.push(stretch);
    }
  }
  return joined
    .map(([start, end]) => ({
      endMs: Math.round(end),
      startMs: Math.round(start),
    }))
    .filter(({ endMs, startMs }) => endMs > startMs);
}

/**
 * `clips`, timed on the timeline `change.previous` lays out, carried onto the
 * one `change.next` lays out. A clip whose footage plays in several places
 * becomes one clip per place, made by `part` from its stretch and its place
 * among them; one with no footage left is dropped. The list itself where
 * nothing moves.
 */
export function retimeRecordingTimelineClips<Clip extends Timed>(
  clips: readonly Clip[],
  change: RecordingTimelineChange,
  part: (clip: Clip, stretch: Timed, index: number) => Clip,
): readonly Clip[] {
  if (change.sourceDurationMs <= 0) return clips;
  const parts = clips.map((clip) => {
    const stretches = retimedStretches(clip, change);
    return stretches.length === 1 &&
      stretches[0].startMs === clip.startMs &&
      stretches[0].endMs === clip.endMs
      ? [clip]
      : stretches.map((stretch, index) => part(clip, stretch, index));
  });
  return parts.every((each, index) => each[0] === clips[index])
    ? clips
    : parts.flat();
}
