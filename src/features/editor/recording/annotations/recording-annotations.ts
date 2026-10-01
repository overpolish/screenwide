// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { freshAnnotationIndex } from "../../annotations/annotation-order";
import {
  AnnotationFrame,
  annotationDrawInMs,
  annotationPathMs,
} from "../../annotations/annotation-pace";
import { Annotation } from "../../annotations/annotations";
import {
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  RecordingTimelineEdit,
} from "../../timeline/editing/recording-timeline-edit";
import { RecordingVideoTrackId } from "../../types";

/** Where a pinned annotation is put at one moment, as a movement in source
 * pixels from where it was drawn. A redaction resized away from the frame it
 * was pinned on also carries how far each edge - left, top, right, bottom -
 * sits outside the box it was drawn as there. One `outOfView` says instead
 * that the content cannot be seen from its moment until the next keyframe. */
type RecordingAnnotationPinKeyframe = {
  dx: number;
  dy: number;
  ms: number;
  edges?: [number, number, number, number];
  outOfView?: boolean;
};

/**
 * An annotation following the content it was placed on. The annotation's own
 * geometry is where it was drawn; each keyframe says where it is at one
 * moment, and the native tracker fills in every other frame. `pinnedMs` is
 * the keyframe the pin was made on, which clearing the corrections keeps.
 */
export type RecordingAnnotationPin = {
  keyframes: RecordingAnnotationPinKeyframe[];
  pinnedMs: number;
};

/** An annotation in source time, attached to one of the recording's two panes.
 * `pathMs` is how long what it draws along a path takes to draw in, paced by
 * the path's length; see `annotation-pace.ts`. */
export type RecordingAnnotationClip = {
  annotation: Annotation;
  endMs: number;
  startMs: number;
  trackId: "primary" | "camera";
  pathMs?: number;
  pin?: RecordingAnnotationPin;
};

/** Whether two clips are ever showing at once, and so drawn one over the
 * other: what a move through the drawing order passes. */
export const recordingAnnotationClipsMeet = (
  a: RecordingAnnotationClip,
  b: RecordingAnnotationClip,
) => a.startMs < b.endMs && b.startMs < a.endMs;

/**
 * The clips with their counters numbered 1, 2, 3 in the order their clips
 * start on the timeline.
 *
 * A counter counts what the viewer sees, and on a timeline that order is time:
 * dragging the second counter's clip in front of the first makes it the first.
 * Only the numbers follow time; where a counter sits in the drawing order is
 * its own, so reordering it renumbers nothing. Counters starting on the same
 * frame keep the order of the numbers they already had, a fresh one - any
 * not among `previous` - after the rest, so a tie never flips as the drawing
 * order changes.
 *
 * The numbering is derived rather than kept, so a drag, a trim, an undo and a
 * delete all land on the same numbers without any of them knowing about
 * counters.
 */
export const renumberedAnnotationClips = (
  clips: RecordingAnnotationClip[],
  previous: RecordingAnnotationClip[],
): RecordingAnnotationClip[] => {
  const known = new Set(previous.map((clip) => clip.annotation.id));
  const order = clips
    .flatMap((clip, index) =>
      clip.annotation.shape.kind === "counter"
        ? [
            {
              fresh: known.has(clip.annotation.id) ? 0 : 1,
              index,
              startMs: clip.startMs,
              value: clip.annotation.shape.value,
            },
          ]
        : [],
    )
    .sort(
      (a, b) =>
        a.startMs - b.startMs ||
        a.fresh - b.fresh ||
        a.value - b.value ||
        a.index - b.index,
    );
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

/** A pin as a native edit left it: dragging a pinned annotation whole gives it
 * a keyframe rather than moving what was drawn. */
export type RecordingAnnotationPinCommit = {
  annotationId: string;
  pin: RecordingAnnotationPin;
};

/** Merges the annotations an edit on the picture reports into their clips,
 * with the pins the same edit left on them. An annotation already in a clip
 * updates that clip, on whichever pane: a group carried together reports its
 * members from both. A fresh annotation joins `trackId`, paced on `frame`,
 * the pane's picture. */
export const mergeRecordingAnnotationClips = ({
  annotations,
  clips,
  edit,
  frame,
  pins = [],
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
  frame?: AnnotationFrame;
  pins?: RecordingAnnotationPinCommit[];
}) => {
  const byId = new Map(
    annotations.map((annotation) => [annotation.id, annotation]),
  );
  const pinById = new Map(
    pins.map(({ annotationId, pin }) => [annotationId, pin]),
  );
  const next = clips.map((clip) => {
    if (!byId.has(clip.annotation.id)) return clip;
    const pin = pinById.get(clip.annotation.id);
    return {
      ...clip,
      annotation: byId.get(clip.annotation.id) ?? clip.annotation,
      ...(pin && clip.pin ? { pin } : {}),
    };
  });
  const known = new Set(next.map((clip) => clip.annotation.id));
  for (const annotation of annotations) {
    if (known.has(annotation.id)) continue;
    next.splice(
      freshAnnotationIndex(
        next.map((clip) => clip.annotation.shape.kind),
        annotation.shape.kind,
      ),
      0,
      recordingAnnotationClipAt({
        annotation,
        edit,
        frame,
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
 * is whole where it was placed - its path's paced time for an arrow drawing
 * itself on `frame`, a fifth of a second for a counter growing into place.
 * Near the start of the recording it takes whatever room there is and the
 * annotation is caught part drawn, which is the only way an annotation can be
 * placed there at all. The clip's end is measured from the playhead as
 * before, so the reach back lengthens the clip rather than sliding it.
 */
export const recordingAnnotationClipAt = ({
  annotation,
  edit,
  frame,
  sourceDurationMs,
  sourcePositionMs,
  trackId = "primary",
}: {
  annotation: Annotation;
  sourceDurationMs: number;
  sourcePositionMs: number;
  edit?: RecordingTimelineEdit | null;
  frame?: AnnotationFrame;
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
  const pathMs = annotationPathMs(annotation, frame);
  const drawInMs = annotationDrawInMs(annotation, pathMs);
  // The reveal plays in output time, so the reach back is measured there and
  // taken back into source time: a cut or a speed change behind the playhead
  // still leaves the annotation whole where it was placed.
  const startMs =
    edit && outputDurationMs > 0
      ? recordingTimelineOutputToSource(
          edit,
          Math.max(0, outputStartMs - drawInMs) / outputDurationMs,
        ) * duration
      : positionMs - drawInMs;
  return {
    annotation,
    endMs: Math.round(Math.min(duration, endMs)),
    ...(pathMs === undefined ? {} : { pathMs }),
    startMs: Math.max(0, Math.round(startMs)),
    trackId,
  };
};
