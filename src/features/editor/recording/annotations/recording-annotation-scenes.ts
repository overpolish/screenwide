// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Annotation, AnnotationShape } from "../../annotations/annotations";

import type { RecordingAnnotationClip } from "./recording-annotations";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";
import type { SceneShape } from "../../../../bindings/SceneShape";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/**
 * An annotation's own place in a scene: moved while the playhead is in that
 * scene, it is drawn there for the scene alone and glides there and back as
 * the scene arrives and leaves. Only the shape's geometry is the scene's;
 * what it says and is stays the annotation's own. The twin of
 * `scene_shape.rs`, which resolves and folds the native edits the same way.
 */
export type { SceneShape };

type ShapeKind = AnnotationShape["kind"];

/** Each kind's geometry, a scene's to change; every other field is what the
 * annotation says and is, and stays its own. The twin of
 * `AnnotationShape::with_geometry_of`. */
const GEOMETRY: Record<ShapeKind, readonly string[]> = {
  arrow: ["start", "control", "end"],
  counter: ["center", "angle"],
  draw: ["points"],
  focus: ["start", "end"],
  highlight: ["start", "end", "bands"],
  image: ["center", "size", "angle", "tilt", "flip"],
  magnify: ["start", "end", "zoom"],
  redact: ["start", "end"],
  shape: ["start", "end"],
  spotlight: ["start", "end"],
  text: ["origin", "controls"],
};

/** `shape` with `geometry`'s geometry: what it says and is kept, where it
 * sits taken from `geometry`. A shape of another kind leaves it whole. */
const withGeometryOf = (
  shape: AnnotationShape,
  geometry: AnnotationShape,
): AnnotationShape => {
  if (shape.kind !== geometry.kind) return shape;
  const keys = GEOMETRY[shape.kind];
  const from = geometry as Record<string, unknown>;
  return Object.fromEntries([
    ...Object.entries(shape).filter(([key]) => !keys.includes(key)),
    ...keys.flatMap((key) =>
      from[key] === undefined ? [] : [[key, from[key]] as const],
    ),
  ]) as AnnotationShape;
};

/** Whether `a` and `b` sit in the same place: the same geometry, whatever
 * else differs. */
export const sameGeometry = (a: AnnotationShape, b: AnnotationShape) => {
  if (a.kind !== b.kind) return false;
  const [one, other] = [a, b] as Record<string, unknown>[];
  return GEOMETRY[a.kind].every(
    (key) => JSON.stringify(one[key]) === JSON.stringify(other[key]),
  );
};

/** `clip`'s annotation as it stands in the scene `sceneId`: its own, with the
 * geometry it was given there, if any. A pinned annotation follows its
 * content instead and keeps its own. */
export const annotationInScene = (
  clip: RecordingAnnotationClip,
  sceneId: string | null,
): Annotation => {
  const placed =
    sceneId && !clip.pin
      ? clip.sceneShapes?.find((each) => each.sceneId === sceneId)
      : undefined;
  return placed
    ? {
        ...clip.annotation,
        shape: withGeometryOf(clip.annotation.shape, placed.shape),
      }
    : clip.annotation;
};

/** Whether `clip`'s annotation has a place of its own in the scene
 * `sceneId`. */
export const isPlacedInScene = (
  clip: RecordingAnnotationClip,
  sceneId: string,
) => clip.sceneShapes?.some((each) => each.sceneId === sceneId) ?? false;

/** `clip` with `edited`, its annotation as an edit made in the scene
 * `sceneId` left it: geometry moved there becomes the scene's, and the rest
 * of the edit, its words or its look, the annotation's own. An edit that
 * leaves the geometry where the scene drew it gives the scene nothing. The
 * twin of `fold_in_scene`. */
export const withEditInScene = (
  clip: RecordingAnnotationClip,
  edited: Annotation,
  sceneId: string,
): RecordingAnnotationClip => {
  const drawn = annotationInScene(clip, sceneId).shape;
  const own = {
    ...edited,
    shape: withGeometryOf(edited.shape, clip.annotation.shape),
  };
  if (sameGeometry(drawn, edited.shape)) return { ...clip, annotation: own };
  const others = (clip.sceneShapes ?? []).filter(
    (each) => each.sceneId !== sceneId,
  );
  return {
    ...clip,
    annotation: own,
    sceneShapes: [...others, { sceneId, shape: edited.shape }],
  };
};

/** `clip` with no place of its own in the scene `sceneId`, back where it is
 * outside every scene. */
export const withoutSceneShape = (
  clip: RecordingAnnotationClip,
  sceneId: string,
): RecordingAnnotationClip => {
  const kept = (clip.sceneShapes ?? []).filter(
    (each) => each.sceneId !== sceneId,
  );
  if (kept.length === (clip.sceneShapes?.length ?? 0)) return clip;
  const { sceneShapes: _dropped, ...rest } = clip;
  return kept.length > 0 ? { ...rest, sceneShapes: kept } : rest;
};

/** `clips` with their scene shapes following `sceneIds`, the scenes there
 * now: a scene gone takes its shapes with it, and one cut in two hands each
 * part the shapes it had (`parts`, each new part's id to the one it was cut
 * from). Unchanged clips are returned as they were. */
export const withSceneShapesFollowing = (
  clips: RecordingAnnotationClip[],
  sceneIds: readonly string[],
  parts: ReadonlyMap<string, string> = new Map(),
): RecordingAnnotationClip[] => {
  const present = new Set(sceneIds);
  const next = clips.map((clip): RecordingAnnotationClip => {
    if (!clip.sceneShapes?.length) return clip;
    const shapes = clip.sceneShapes;
    const carried = [...parts].flatMap(([part, from]) => {
      const shape = shapes.find((each) => each.sceneId === from);
      return shape && !shapes.some((each) => each.sceneId === part)
        ? [{ sceneId: part, shape: shape.shape }]
        : [];
    });
    const kept = [...shapes, ...carried].filter((each) =>
      present.has(each.sceneId),
    );
    if (carried.length === 0 && kept.length === shapes.length) return clip;
    const { sceneShapes: _dropped, ...rest } = clip;
    return kept.length > 0 ? { ...rest, sceneShapes: kept } : rest;
  });
  return next.every((clip, index) => clip === clips[index]) ? clips : next;
};

/** `edit` with the scenes `sceneClips`: a scene taken away takes the places
 * annotations had in it. */
export const withSceneClips = (
  edit: RecordingTimelineEdit,
  sceneClips: RecordingSceneClip[],
): RecordingTimelineEdit => ({
  ...edit,
  annotationClips:
    edit.annotationClips &&
    withSceneShapesFollowing(
      edit.annotationClips,
      sceneClips.map((clip) => clip.id),
    ),
  sceneClips,
});
