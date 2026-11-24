// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { reshapedBoxes } from "./format-document";
import {
  type FormatContext,
  isShownIn,
  layoutKeys,
  proportionKey,
} from "./format-scope";
import { formatInHand, formatsOfShape } from "./recording-formats";

import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { FrameFormats } from "../../../../bindings/FrameFormats";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";
import type { SceneFormatLayout } from "../../../../bindings/SceneFormatLayout";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

type Size = { height: number; width: number };

/** Every shape and its formats; the format `id` (the one in hand where it
 * names none of them) and its shape's canvas, the shape in hand's being
 * `canvas` and the others' the one each keeps aside; and the shared canvas
 * the edit's own layouts were made on: the main shape's, the first. */
export const formatContext = (
  formats: FrameFormats,
  canvas: Size,
  id?: string,
): { base: Size; canvas: Size; context: FormatContext } => {
  const shapes = formats.list.map((shape, index) => {
    const size =
      shape.id === formats.active
        ? canvas
        : (shape.look?.recordingOutput.primary ?? canvas);
    return {
      // The main canvas keeps its own: no platform shares its layouts.
      annotationKey: index === 0 ? shape.id : proportionKey(size),
      formats: formatsOfShape(shape),
      id: shape.id,
      inHand: formatInHand(shape),
      size,
    };
  });
  // A recording's formats always hold the shape in hand.
  const target =
    shapes.find((shape) => id !== undefined && shape.formats.includes(id)) ??
    shapes.find((shape) => shape.id === formats.active) ??
    shapes[0];
  const isNamed = id !== undefined && target.formats.includes(id);
  return {
    base: shapes[0].size,
    canvas: { height: target.size.height, width: target.size.width },
    context: {
      annotationKey: target.annotationKey,
      id: isNamed ? id : target.inHand,
      shape: target.id,
      shapes: shapes.map((shape) => ({
        annotationKey: shape.annotationKey,
        formats: shape.formats,
        id: shape.id,
      })),
    },
  };
};

/** The parts of a scene each format can lay out for itself. */
export const SCENE_LAYOUT_KEYS = [
  "preset",
  "layoutBefore",
  "variant",
  "boxes",
  "shapes",
  "inset",
  "radius",
  "rotation",
  "screen",
  "camera",
] as const satisfies readonly (keyof SceneFormatLayout &
  keyof RecordingSceneClip)[];

export const sceneLayoutOf = (clip: RecordingSceneClip): SceneFormatLayout => {
  const layout: SceneFormatLayout = { preset: clip.preset };
  for (const key of SCENE_LAYOUT_KEYS) {
    if (clip[key] !== undefined) Object.assign(layout, { [key]: clip[key] });
  }
  return layout;
};

/** `clip` laid out as `layout` has it, keeping its timing and the rest. */
export const withSceneLayout = (
  clip: RecordingSceneClip,
  layout: SceneFormatLayout,
): RecordingSceneClip => {
  const next: RecordingSceneClip = { ...clip };
  for (const key of SCENE_LAYOUT_KEYS) {
    if (layout[key] === undefined) Reflect.deleteProperty(next, key);
    else Object.assign(next, { [key]: layout[key] });
  }
  return next;
};

/** `clip` changing nothing in the picture: the recording's own layout,
 * background, cursor and camera, for the time it holds. */
export const neutralScene = ({
  auto,
  endMs,
  id,
  startMs,
}: RecordingSceneClip): RecordingSceneClip => ({
  auto,
  endMs,
  id,
  preset: "full",
  startMs,
});

/**
 * The format in hand's timeline, worked out from the one edit every format
 * shares and what `edits` does differently: what does not show in it left
 * out, and each scene, annotation and caption track laid out as its shape
 * has it. A custom scene with no layout of the shape's own keeps its boxes'
 * middles and shapes on the shape's canvas, reshaped from the shared
 * canvas `base` it was made on, `canvas`.
 */
export function resolvedTimelineEdit(
  edit: RecordingTimelineEdit,
  edits: FormatEdits | undefined,
  {
    base,
    canvas,
    context,
  }: { base: Size; canvas: Size; context: FormatContext },
): RecordingTimelineEdit {
  const keys = layoutKeys(context);
  const own = <T>(layouts: Partial<Record<string, T>> | undefined) =>
    keys.map((key) => layouts?.[key]).find((layout) => layout !== undefined);
  const reshaped =
    base.width * canvas.height !== canvas.width * base.height &&
    base.height > 0 &&
    canvas.height > 0;
  const sceneClips = edit.sceneClips?.map((clip) => {
    const formats = edits?.scenes?.[clip.id];
    // A scene left out of this format keeps its slot on the strip as a plain
    // full scene, which changes nothing in the picture: scenes in different
    // formats share their time and never overlap.
    if (!isShownIn(formats?.showIn, context)) return neutralScene(clip);
    const layout = own(formats?.layouts);
    if (layout) return withSceneLayout(clip, layout);
    return reshaped && clip.boxes
      ? { ...clip, boxes: reshapedBoxes(clip.boxes, base, canvas) }
      : clip;
  });
  const annotationClips = edit.annotationClips
    ?.filter((clip) =>
      isShownIn(edits?.annotations?.[clip.annotation.id]?.showIn, context),
    )
    .map((clip) => {
      const annotation = own(edits?.annotations?.[clip.annotation.id]?.layouts);
      return annotation ? { ...clip, annotation } : clip;
    });
  const audioClips = edit.audioClips?.filter((clip) =>
    isShownIn(edits?.audio?.[clip.id]?.showIn, context),
  );
  const captions = edit.captions && {
    ...edit.captions,
    ...own(edits?.captions?.layouts),
    enabled:
      edit.captions.enabled && isShownIn(edits?.captions?.showIn, context),
  };
  return {
    ...edit,
    ...(annotationClips ? { annotationClips } : {}),
    ...(audioClips ? { audioClips } : {}),
    ...(captions ? { captions } : {}),
    ...(sceneClips ? { sceneClips } : {}),
  };
}
