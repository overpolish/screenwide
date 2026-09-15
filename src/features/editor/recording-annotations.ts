// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Annotation } from "./annotations";
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

/** Creates the initial three output seconds of an annotation clip. */
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
  const startMs = Math.max(
    0,
    Math.min(duration - 1, Math.round(sourcePositionMs)),
  );
  const outputDurationMs = edit
    ? recordingTimelineRetainedDuration(edit) * duration
    : duration;
  const outputStartMs = edit
    ? recordingTimelineSourceToOutput(
        edit,
        duration > 0 ? startMs / duration : 0,
      ) * outputDurationMs
    : startMs;
  const outputEndMs = Math.min(outputDurationMs, outputStartMs + 3_000);
  const endMs = edit
    ? Math.max(
        startMs + (startMs < duration ? 1 : 0),
        recordingTimelineOutputToSource(
          edit,
          outputDurationMs > 0 ? outputEndMs / outputDurationMs : 1,
        ) * duration,
      )
    : Math.min(duration, startMs + 3_000);
  return {
    annotation,
    endMs: Math.round(Math.min(duration, endMs)),
    startMs,
    trackId,
  };
};
