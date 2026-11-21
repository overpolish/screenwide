// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { drawnCropRect } from "../../screenshot/screenshot-crop";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import {
  heldSceneCrop,
  holdSceneCrop,
  releaseSceneCrop,
} from "./recording-scene-crop-hold";
import { customRecordingSceneClip } from "./recording-scene-custom";
import { croppedCustomPane } from "./recording-scene-custom-crop";
import { SceneRect } from "./recording-scene-geometry";
import { PaneName, ScenePane } from "./recording-scene-gesture-edits";
import { SceneCamera } from "./recording-scene-placement";
import { RecordingSceneClip } from "./recording-scenes";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";

/** A gesture that edits one scene, event by event. */
export type SceneEditGesture = {
  clipId: string;
  clips: RecordingSceneClip[];
  /** Each event's edit of the clip, from the clip as the gesture found it;
   * null leaves it as it was. */
  edit: (
    clip: RecordingSceneClip,
    event: RecordingSelectionGestureEvent,
  ) => RecordingSceneClip | null;
  kind: "edit";
  /** Puts back what the gesture changed beside the clip, when it is
   * cancelled. */
  onCancel?: () => void;
};

/**
 * The crop tool on a scene's pane, which crops it in place, the picture
 * staying where it sits: a custom scene's box takes the crop, and a preset
 * keeps its layout, laying the pane out at the crop's shape (`shapes`), so
 * the crop never makes it custom. Laid out afresh at every step, a preset's
 * pane would slide away from the edges held still under the pointer, so the
 * preview draws it as a custom scene where it stood until the crop tool
 * leaves the scene (`recording-scene-crop-hold.ts`).
 */
export function sceneCropGesture({
  anchored,
  camera,
  canvas,
  clips,
  event,
  keptSpot,
  output,
  pane,
  paneName,
  scene,
}: {
  /** Whether the window is held about its middle, the crop of a camera
   * that follows the face (`cropAnchored`). */
  anchored: boolean;
  camera: SceneCamera | null;
  /** The canvas in output pixels; the native gesture reports its travel as
   * shares of it. */
  canvas: { height: number; width: number };
  clips: RecordingSceneClip[];
  event: RecordingSelectionGestureEvent;
  keptSpot: (framing: ScenePane["framing"]) => ScenePane["framing"];
  output: ScreenshotOutputSettings;
  pane: ScenePane;
  paneName: PaneName;
  scene: RecordingSceneClip;
}): SceneEditGesture {
  const { boxes } = scene;
  const anchor = {
    x: pane.box.x + event.deltaX * canvas.width,
    y: pane.box.y + event.deltaY * canvas.height,
  };
  const wasHeld = boxes ? null : heldSceneCrop(scene);
  const held = boxes
    ? null
    : (wasHeld ?? customRecordingSceneClip({ camera, clip: scene, output }));
  return {
    clipId: scene.id,
    clips,
    edit: (_clip, next) => {
      if (
        next.operation !== "cropDraw" &&
        next.operation !== "cropMove" &&
        next.operation !== "cropResize"
      )
        return null;
      // A press that takes a preset's crop without moving it leaves the
      // scene as it was.
      if (
        !boxes &&
        next.operation !== "cropDraw" &&
        next.deltaX === 0 &&
        next.deltaY === 0 &&
        next.scale === 1
      )
        return null;
      const cropped = croppedCustomPane(pane, {
        canvas,
        // An anchored window grows about its middle.
        moving: next.operation === "cropMove" || anchored ? 0 : next.edges & 15,
        next: customCropWindow(pane.box, {
          anchor,
          anchored,
          delta: {
            x: next.deltaX * canvas.width,
            y: next.deltaY * canvas.height,
          },
          edges: next.edges,
          operation: next.operation,
        }),
        // A preset's window picks a part of the picture, wherever it lies.
        onCanvas: Boolean(boxes),
        slides: next.operation === "cropMove",
      });
      const framing = { [paneName]: keptSpot(cropped.framing) };
      if (boxes)
        return {
          ...scene,
          boxes: { ...boxes, [paneName]: cropped.box },
          ...framing,
        };
      const shape =
        (cropped.box.width * canvas.width) /
        Math.max(cropped.box.height * canvas.height, Number.EPSILON);
      const shaped = {
        ...scene,
        shapes: { ...scene.shapes, [paneName]: shape },
        ...framing,
      };
      if (held?.boxes)
        holdSceneCrop(shaped, {
          ...held,
          boxes: { ...held.boxes, [paneName]: cropped.box },
          ...framing,
        });
      return shaped;
    },
    kind: "edit",
    ...(boxes
      ? {}
      : {
          onCancel: () => {
            if (wasHeld) holdSceneCrop(scene, wasHeld);
            else releaseSceneCrop();
          },
        }),
  };
}

/**
 * The crop window a crop gesture leaves on `pane`, in output pixels, from the
 * box as the gesture found it. `delta` is the gesture's travel in output
 * pixels; a move carries the window, a resize carries the edges `edges`
 * names, or, `anchored` about its middle, grows each side by `delta` (the
 * right and bottom edges' travel), and a draw spans from `anchor`, a point
 * in output pixels.
 */
export const customCropWindow = (
  box: SceneRect,
  {
    anchor,
    anchored = false,
    delta,
    edges,
    operation,
  }: {
    anchor: { x: number; y: number };
    delta: { x: number; y: number };
    edges: number;
    operation: "cropDraw" | "cropMove" | "cropResize";
    anchored?: boolean;
  },
): SceneRect => {
  if (operation === "cropMove")
    return { ...box, x: box.x + delta.x, y: box.y + delta.y };
  if (operation === "cropDraw")
    return drawnCropRect(anchor, {
      deltaX: delta.x,
      deltaY: delta.y,
      edges,
    });
  if (anchored)
    return {
      height: box.height + delta.y * 2,
      width: box.width + delta.x * 2,
      x: box.x - delta.x,
      y: box.y - delta.y,
    };
  const left = box.x + ((edges & 1) !== 0 ? delta.x : 0);
  const right = box.x + box.width + ((edges & 2) !== 0 ? delta.x : 0);
  const top = box.y + ((edges & 4) !== 0 ? delta.y : 0);
  const bottom = box.y + box.height + ((edges & 8) !== 0 ? delta.y : 0);
  return { height: bottom - top, width: right - left, x: left, y: top };
};
