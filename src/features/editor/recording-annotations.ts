// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Annotation, ANNOTATION_DRAW_IN_MS } from "./annotations";
import {
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  RecordingTimelineEdit,
} from "./recording-timeline-edit";
import { RecordingVideoTrackId } from "./types";

/** A mark in source time, attached to one of the recording's two panes. */
export type RecordingAnnotationClip = {
  annotation: Annotation;
  endMs: number;
  startMs: number;
  trackId: "primary" | "camera";
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
 * Creates the initial three output seconds of an annotation clip, reaching a
 * draw-in back before the playhead so an animated mark has finished drawing
 * itself where it was placed. Near the start of the recording it takes
 * whatever room there is and the mark is caught part drawn, which is the only
 * way a mark can be placed there at all. The clip's end is measured from the
 * playhead as before, so the reach back lengthens the clip rather than
 * sliding it.
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
    startMs: Math.max(0, positionMs - ANNOTATION_DRAW_IN_MS),
    trackId,
  };
};
