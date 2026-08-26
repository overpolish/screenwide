// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type RecordingTimelineSegment = {
  id: number;
  /** Normalized position in the original recording, before magnetic edits. */
  sourceEnd: number;
  /** Normalized position in the original recording, before magnetic edits. */
  sourceStart: number;
};

export type RecordingTimelineEdit = {
  artifactId: number;
  nextSegmentId: number;
  segments: RecordingTimelineSegment[];
};

export type RecordingTimelineTrimEdge = "end" | "start";

export type RecordingTimelineLayoutSegment = RecordingTimelineSegment & {
  /** Normalized position in the retained, magnetic output timeline. */
  outputEnd: number;
  /** Normalized position in the retained, magnetic output timeline. */
  outputStart: number;
};

export const createRecordingTimelineEdit = (
  artifactId: number,
): RecordingTimelineEdit => ({
  artifactId,
  nextSegmentId: 1,
  segments: [{ id: 0, sourceEnd: 1, sourceStart: 0 }],
});

export const recordingTimelineRetainedDuration = (
  edit: RecordingTimelineEdit,
) =>
  edit.segments.reduce(
    (total, segment) => total + segment.sourceEnd - segment.sourceStart,
    0,
  );

export function layoutRecordingTimelineSegments(
  edit: RecordingTimelineEdit,
): RecordingTimelineLayoutSegment[] {
  const retainedDuration = recordingTimelineRetainedDuration(edit);
  let retainedStart = 0;
  return edit.segments.map((segment, index) => {
    const duration = segment.sourceEnd - segment.sourceStart;
    const outputStart = retainedStart / retainedDuration;
    retainedStart += duration;
    return {
      ...segment,
      outputEnd:
        index === edit.segments.length - 1
          ? 1
          : retainedStart / retainedDuration,
      outputStart,
    };
  });
}

export function recordingTimelineOutputToSource(
  edit: RecordingTimelineEdit,
  outputPosition: number,
): number {
  const retainedDuration = recordingTimelineRetainedDuration(edit);
  const target = Math.max(0, Math.min(1, outputPosition)) * retainedDuration;
  let retainedStart = 0;
  for (const segment of edit.segments) {
    const duration = segment.sourceEnd - segment.sourceStart;
    if (target <= retainedStart + duration)
      return segment.sourceStart + target - retainedStart;
    retainedStart += duration;
  }
  return edit.segments[edit.segments.length - 1]?.sourceEnd ?? 0;
}

export function recordingTimelineSourceToOutput(
  edit: RecordingTimelineEdit,
  sourcePosition: number,
): number {
  const retainedDuration = recordingTimelineRetainedDuration(edit);
  let retainedStart = 0;
  for (const segment of edit.segments) {
    if (sourcePosition <= segment.sourceStart)
      return retainedStart / retainedDuration;
    if (sourcePosition <= segment.sourceEnd)
      return (
        (retainedStart + sourcePosition - segment.sourceStart) /
        retainedDuration
      );
    retainedStart += segment.sourceEnd - segment.sourceStart;
  }
  return 1;
}

/** Keeps absolute magnetic time stable while changing the retained ranges. */
export function remapRecordingTimelinePosition(
  previous: RecordingTimelineEdit,
  next: RecordingTimelineEdit,
  sourcePosition: number,
) {
  const previousOutputTime =
    recordingTimelineSourceToOutput(previous, sourcePosition) *
    recordingTimelineRetainedDuration(previous);
  const nextDuration = recordingTimelineRetainedDuration(next);
  const outputPosition = Math.max(
    0,
    Math.min(1, previousOutputTime / nextDuration),
  );
  return {
    outputPosition,
    sourcePosition: recordingTimelineOutputToSource(next, outputPosition),
  };
}

/** Removes one linked segment while preserving at least one output range. */
export function deleteRecordingTimelineSegment(
  edit: RecordingTimelineEdit,
  segmentId: number,
): RecordingTimelineEdit {
  if (edit.segments.length <= 1) return edit;
  const segments = edit.segments.filter((segment) => segment.id !== segmentId);
  return segments.length === edit.segments.length
    ? edit
    : { ...edit, segments };
}

/** Clamps a trim target to the position `trimRecordingTimelineSegment` accepts. */
export function clampRecordingTimelineTrimPosition(
  edit: RecordingTimelineEdit,
  {
    edge,
    minimumDuration,
    segmentId,
    sourcePosition,
  }: {
    edge: RecordingTimelineTrimEdge;
    minimumDuration: number;
    segmentId: number;
    sourcePosition: number;
  },
): number {
  const index = edit.segments.findIndex((segment) => segment.id === segmentId);
  if (index === -1) return sourcePosition;
  const segment = edit.segments[index];
  const minimum = Math.max(0, minimumDuration);
  if (edge === "start") {
    const lower = edit.segments[index - 1]?.sourceEnd ?? 0;
    return Math.max(
      lower,
      Math.min(segment.sourceEnd - minimum, sourcePosition),
    );
  }
  const upper = edit.segments[index + 1]?.sourceStart ?? 1;
  return Math.min(
    upper,
    Math.max(segment.sourceStart + minimum, sourcePosition),
  );
}

/**
 * Moves one retained edge without overlapping the neighbouring source range.
 * Source omitted between neighbours remains available, so dragging the edge
 * back out can restore a trim or a previously deleted segment.
 */
export function trimRecordingTimelineSegment(
  edit: RecordingTimelineEdit,
  {
    edge,
    minimumDuration,
    segmentId,
    sourcePosition,
  }: {
    edge: RecordingTimelineTrimEdge;
    minimumDuration: number;
    segmentId: number;
    sourcePosition: number;
  },
): RecordingTimelineEdit {
  if (!Number.isFinite(sourcePosition)) return edit;
  const index = edit.segments.findIndex((segment) => segment.id === segmentId);
  if (index === -1) return edit;
  const segment = edit.segments[index];
  const clamped = clampRecordingTimelineTrimPosition(edit, {
    edge,
    minimumDuration,
    segmentId,
    sourcePosition,
  });
  const next =
    edge === "start"
      ? { ...segment, sourceStart: clamped }
      : { ...segment, sourceEnd: clamped };
  if (
    next.sourceStart === segment.sourceStart &&
    next.sourceEnd === segment.sourceEnd
  )
    return edit;
  const segments = [...edit.segments];
  segments[index] = next;
  return { ...edit, segments };
}

/** Snaps a normalized source position to the nearest encoded video frame. */
export function snapRecordingTimelinePosition(
  sourcePosition: number,
  durationMs: number,
  framesPerSecond: number | null,
): number {
  if (!Number.isFinite(sourcePosition)) return 0;
  const position = Math.max(0, Math.min(1, sourcePosition));
  if (position === 0 || position === 1) return position;
  if (!Number.isFinite(durationMs) || durationMs <= 0) return position;
  const frameDurationMs =
    framesPerSecond !== null &&
    Number.isFinite(framesPerSecond) &&
    framesPerSecond > 0
      ? 1_000 / framesPerSecond
      : 1;
  const snappedMs =
    Math.round((position * durationMs) / frameDurationMs) * frameDurationMs;
  return Math.max(0, Math.min(1, snappedMs / durationMs));
}

/**
 * Splits the source range containing `sourcePosition`. Returning the original
 * object for an edge or existing boundary keeps no-op cuts out of undo history.
 */
export function cutRecordingTimeline(
  edit: RecordingTimelineEdit,
  sourcePosition: number,
): RecordingTimelineEdit {
  if (!Number.isFinite(sourcePosition)) return edit;
  const segmentIndex = edit.segments.findIndex(
    (segment) =>
      sourcePosition > segment.sourceStart &&
      sourcePosition < segment.sourceEnd,
  );
  if (segmentIndex === -1) return edit;

  const segment = edit.segments[segmentIndex];
  return {
    ...edit,
    nextSegmentId: edit.nextSegmentId + 1,
    segments: [
      ...edit.segments.slice(0, segmentIndex),
      { ...segment, sourceEnd: sourcePosition },
      {
        id: edit.nextSegmentId,
        sourceEnd: segment.sourceEnd,
        sourceStart: sourcePosition,
      },
      ...edit.segments.slice(segmentIndex + 1),
    ],
  };
}
