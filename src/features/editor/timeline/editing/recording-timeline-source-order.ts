// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Segments play in list order, which a reordered timeline leaves out of
// source order. These answer where source time sits on such a timeline; the
// twins of `output_at_us` and `output_end_at_us` in the `timeline` crate.

import type { RecordingTimelineSegment } from "../recording-timeline-types";

const EPSILON = 1e-9;

const rateOf = (segment: RecordingTimelineSegment) => segment.playbackRate ?? 1;

const outputLengthOf = (segment: RecordingTimelineSegment) =>
  (segment.sourceEnd - segment.sourceStart) / rateOf(segment);

const totalOutputLength = (segments: readonly RecordingTimelineSegment[]) =>
  segments.reduce((sum, segment) => sum + outputLengthOf(segment), 0);

/**
 * The segment a moment the timeline cut away belongs after: the one ending
 * latest at or before `sourcePosition`, where restoring the moment puts it
 * back. -1 when none ends before it.
 */
export function sourcePredecessorIndex(
  segments: readonly RecordingTimelineSegment[],
  sourcePosition: number,
): number {
  let found = -1;
  segments.forEach((segment, index) => {
    if (
      segment.sourceEnd <= sourcePosition + EPSILON &&
      (found < 0 || segment.sourceEnd > segments[found].sourceEnd)
    )
      found = index;
  });
  return found;
}

/** The segment earliest in the source, or -1 with none. */
export function earliestSourceIndex(
  segments: readonly RecordingTimelineSegment[],
): number {
  let found = -1;
  segments.forEach((segment, index) => {
    if (found < 0 || segment.sourceStart < segments[found].sourceStart)
      found = index;
  });
  return found;
}

/**
 * Where `sourcePosition` lands, in output time at the segments' own scale
 * (the sum of their played lengths). `edge` says which segment a moment
 * shared by two takes: a stretch starting there starts in the segment that
 * begins there, one ending there ends with the segment that ends there.
 * A moment cut away lands where its source predecessor ends, or at the start
 * of the segment earliest in the source when none comes before it.
 */
export function retainedOutputAt(
  segments: readonly RecordingTimelineSegment[],
  sourcePosition: number,
  edge: "end" | "start",
): number {
  const starts: number[] = [];
  let before = 0;
  for (const segment of segments) {
    const holds =
      edge === "start"
        ? segment.sourceStart <= sourcePosition &&
          sourcePosition < segment.sourceEnd
        : segment.sourceStart < sourcePosition &&
          sourcePosition <= segment.sourceEnd;
    if (holds)
      return before + (sourcePosition - segment.sourceStart) / rateOf(segment);
    starts.push(before);
    before += outputLengthOf(segment);
  }
  const predecessor = sourcePredecessorIndex(segments, sourcePosition);
  if (predecessor >= 0)
    return starts[predecessor] + outputLengthOf(segments[predecessor]);
  const earliest = earliestSourceIndex(segments);
  return earliest >= 0 ? starts[earliest] : 0;
}

/**
 * Where `sourcePosition` plays, from 0 to 1, for a position the player
 * reports: where two segments share the moment, ending one and starting
 * the other, the side nearer `nearOutput`, the playhead it moves on from.
 */
export function recordingTimelineSourceToOutputNear(
  segments: readonly RecordingTimelineSegment[],
  sourcePosition: number,
  nearOutput: number,
): number {
  const total = totalOutputLength(segments);
  if (total <= 0) return 0;
  const start = retainedOutputAt(segments, sourcePosition, "start") / total;
  const end = retainedOutputAt(segments, sourcePosition, "end") / total;
  return Math.abs(end - nearOutput) < Math.abs(start - nearOutput)
    ? end
    : start;
}

/**
 * How far the segment at `index` may grow without playing source another
 * segment already plays: from the end of the one before it in the source to
 * the start of the one after it, or the recording's ends.
 */
export function freeSourceBounds(
  segments: readonly RecordingTimelineSegment[],
  index: number,
): { lower: number; upper: number } {
  const segment = segments[index];
  let lower = 0;
  let upper = 1;
  segments.forEach((other, otherIndex) => {
    if (otherIndex === index) return;
    if (other.sourceEnd <= segment.sourceStart + EPSILON)
      lower = Math.max(lower, other.sourceEnd);
    else if (other.sourceStart >= segment.sourceEnd - EPSILON)
      upper = Math.min(upper, other.sourceStart);
  });
  return { lower, upper };
}

/**
 * The source moment that starts playing at `outputPosition`, from 0 to 1.
 * Where one segment ends and the next begins, the next one's first moment:
 * on a reordered timeline the one ending there may sit anywhere in the
 * source, so a stretch starting at the join starts in the segment after it.
 */
export function recordingTimelineOutputToSourceStart(
  segments: readonly RecordingTimelineSegment[],
  outputPosition: number,
): number {
  const total = totalOutputLength(segments);
  const target = Math.max(0, Math.min(1, outputPosition)) * total;
  let before = 0;
  for (const segment of segments) {
    const length = outputLengthOf(segment);
    // A join reached by dividing and multiplying back can fall a hair short.
    if (target + EPSILON * total < before + length)
      return (
        segment.sourceStart + Math.max(0, target - before) * rateOf(segment)
      );
    before += length;
  }
  return segments.length > 0 ? segments[segments.length - 1].sourceEnd : 0;
}
