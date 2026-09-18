// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Annotation, annotationDrawInMs } from "./annotations";
import {
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  RecordingTimelineEdit,
} from "./recording-timeline-edit";
import { RecordingVideoTrackId } from "./types";

/** An annotation in source time, attached to one of the recording's two panes.
 */
export type RecordingAnnotationClip = {
  annotation: Annotation;
  endMs: number;
  startMs: number;
  trackId: "primary" | "camera";
};

/**
 * The clips with their counters numbered 1, 2, 3 in the order they appear on
 * the timeline.
 *
 * A counter counts what the viewer sees, and on a timeline that order is time:
 * dragging the second counter's clip in front of the first makes it the first.
 * Two clips starting on the same frame keep the order they were drawn in, which
 * is the order they are stored in - the earlier annotation wins the lower
 * number rather than the pair flickering between them.
 *
 * The numbering is derived rather than kept, so a drag, a trim, an undo and a
 * delete all land on the same numbers without any of them knowing about
 * counters.
 */
export const renumberedAnnotationClips = (
  clips: RecordingAnnotationClip[],
): RecordingAnnotationClip[] => {
  const order = clips
    .map((clip, index) => ({ clip, index }))
    .filter(({ clip }) => clip.annotation.shape.kind === "counter")
    .sort((a, b) => a.clip.startMs - b.clip.startMs || a.index - b.index);
  const values = new Map(order.map(({ index }, place) => [index, place + 1]));
  const numbered = clips.map((clip, index) => {
    const value = values.get(index);
    const shape = clip.annotation.shape;
    if (
      value === undefined ||
      shape.kind !== "counter" ||
      shape.value === value
    )
      return clip;
    return {
      ...clip,
      annotation: { ...clip.annotation, shape: { ...shape, value } },
    };
  });
  return numbered.every((clip, index) => clip === clips[index])
    ? clips
    : numbered;
};

/** Merges the annotations currently drawn by a native pane into its clips. */
export const mergeRecordingAnnotationClips = ({
  annotations,
  clips,
  edit,
  positionMs,
  sourceDurationMs,
  trackId,
}: {
  annotations: Annotation[];
  clips: RecordingAnnotationClip[];
  edit: RecordingTimelineEdit;
  positionMs: number;
  sourceDurationMs: number;
  trackId: RecordingVideoTrackId;
}) => {
  const byId = new Map(
    annotations.map((annotation) => [annotation.id, annotation]),
  );
  const next = clips.map((clip) =>
    clip.trackId === trackId && byId.has(clip.annotation.id)
      ? {
          ...clip,
          annotation: byId.get(clip.annotation.id) ?? clip.annotation,
        }
      : clip,
  );
  const known = new Set(next.map((clip) => clip.annotation.id));
  for (const annotation of annotations) {
    if (!known.has(annotation.id))
      next.push(
        recordingAnnotationClipAt({
          annotation,
          edit,
          sourceDurationMs,
          sourcePositionMs: positionMs,
          trackId,
        }),
      );
  }
  return next;
};

/**
 * Creates the initial three output seconds of an annotation clip, reaching the
 * annotation's own arrival back before the playhead so an animated annotation
 * is whole where it was placed - a second for an arrow drawing itself, a fifth
 * of one for a counter growing into place. Near the start of the recording it
 * takes whatever room there is and the annotation is caught part drawn, which
 * is the only way an annotation can be placed there at all. The clip's end is
 * measured from the playhead as before, so the reach back lengthens the clip
 * rather than sliding it.
 */
export const recordingAnnotationClipAt = ({
  annotation,
  edit,
  sourceDurationMs,
  sourcePositionMs,
  trackId = "primary",
}: {
  annotation: Annotation;
  sourceDurationMs: number;
  sourcePositionMs: number;
  edit?: RecordingTimelineEdit | null;
  trackId?: RecordingAnnotationClip["trackId"];
}): RecordingAnnotationClip => {
  const duration = Math.max(1, Math.round(sourceDurationMs));
  const positionMs = Math.max(
    0,
    Math.min(duration - 1, Math.round(sourcePositionMs)),
  );
  const outputDurationMs = edit
    ? recordingTimelineRetainedDuration(edit) * duration
    : duration;
  const outputStartMs = edit
    ? recordingTimelineSourceToOutput(
        edit,
        duration > 0 ? positionMs / duration : 0,
      ) * outputDurationMs
    : positionMs;
  const outputEndMs = Math.min(outputDurationMs, outputStartMs + 3_000);
  const endMs = edit
    ? Math.max(
        positionMs + (positionMs < duration ? 1 : 0),
        recordingTimelineOutputToSource(
          edit,
          outputDurationMs > 0 ? outputEndMs / outputDurationMs : 1,
        ) * duration,
      )
    : Math.min(duration, positionMs + 3_000);
  return {
    annotation,
    endMs: Math.round(Math.min(duration, endMs)),
    // The reveal runs on source time, so the reach back is source time too.
    startMs: Math.max(0, positionMs - annotationDrawInMs(annotation)),
    trackId,
  };
};
