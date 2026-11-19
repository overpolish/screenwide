// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { freshAnnotationIndex } from "../../annotations/annotation-order";
import { AnnotationFrame } from "../../annotations/annotation-pace";
import { Annotation } from "../../annotations/annotations";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { RecordingVideoTrackId } from "../../types";

import {
  RecordingAnnotationClip,
  recordingAnnotationClipAt,
  RecordingAnnotationPin,
} from "./recording-annotations";

import type { SceneShape } from "./recording-annotation-scenes";

/** A pin as a native edit left it: dragging a pinned annotation whole gives it
 * a keyframe rather than moving what was drawn. */
export type RecordingAnnotationPinCommit = {
  annotationId: string;
  pin: RecordingAnnotationPin;
};

/** An annotation's places in the scenes as a native edit inside a scene left
 * them. */
export type RecordingAnnotationSceneShapesCommit = {
  annotationId: string;
  sceneShapes: SceneShape[];
};

/** A copy a carry with the duplicate modifier took, and whose copy it is. */
export type RecordingAnnotationCopyCommit = {
  annotationId: string;
  originalId: string;
};

/** `clip` with the places in the scenes an edit left it, where it left any. */
const withPlaces = (
  clip: RecordingAnnotationClip,
  places: SceneShape[] | undefined,
): RecordingAnnotationClip => {
  if (places === undefined) return clip;
  const { sceneShapes: _kept, ...rest } = clip;
  return places.length > 0 ? { ...rest, sceneShapes: places } : rest;
};

/** Merges the annotations an edit on the picture reports into their clips,
 * with the pins and scene places the same edit left on them. An annotation
 * already in a clip updates that clip, on whichever pane: a group carried
 * together reports its members from both. A copy taken with the duplicate
 * modifier goes just above its original, timed as it is and following no
 * content. Any other fresh annotation joins `trackId`, paced on `frame`, the
 * pane's picture. */
export const mergeRecordingAnnotationClips = ({
  annotations,
  clips,
  copies = [],
  edit,
  frame,
  pins = [],
  sceneShapes = [],
  sourceDurationMs,
  sourcePositionMs,
  trackId,
}: {
  annotations: Annotation[];
  clips: RecordingAnnotationClip[];
  edit: RecordingTimelineEdit;
  sourceDurationMs: number;
  /** Where the playhead was in the recording, in milliseconds. */
  sourcePositionMs: number;
  trackId: RecordingVideoTrackId;
  copies?: RecordingAnnotationCopyCommit[];
  frame?: AnnotationFrame;
  pins?: RecordingAnnotationPinCommit[];
  sceneShapes?: RecordingAnnotationSceneShapesCommit[];
}) => {
  const byId = new Map(
    annotations.map((annotation) => [annotation.id, annotation]),
  );
  const pinById = new Map(
    pins.map(({ annotationId, pin }) => [annotationId, pin]),
  );
  const placesById = new Map(
    sceneShapes.map(({ annotationId, sceneShapes: shapes }) => [
      annotationId,
      shapes,
    ]),
  );
  const originalOf = new Map(
    copies.map(({ annotationId, originalId }) => [annotationId, originalId]),
  );
  const next = clips.map((clip) => {
    if (!byId.has(clip.annotation.id)) return clip;
    const pin = pinById.get(clip.annotation.id);
    return {
      ...withPlaces(clip, placesById.get(clip.annotation.id)),
      annotation: byId.get(clip.annotation.id) ?? clip.annotation,
      ...(pin && clip.pin ? { pin } : {}),
    };
  });
  const known = new Set(next.map((clip) => clip.annotation.id));
  for (const annotation of annotations) {
    if (known.has(annotation.id)) continue;
    const original = originalOf.get(annotation.id);
    const at = next.findIndex((clip) => clip.annotation.id === original);
    if (at >= 0) {
      const { pin: _pin, ...free } = next[at];
      next.splice(at + 1, 0, {
        ...withPlaces(free, placesById.get(annotation.id)),
        annotation,
      });
      continue;
    }
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
        sourcePositionMs,
        trackId,
      }),
    );
  }
  return next;
};
