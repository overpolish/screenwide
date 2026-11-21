// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import {
  heldSceneCrop,
  holdSceneCrop,
  releaseSceneCrop,
} from "./recording-scene-crop-hold";
import { customRecordingSceneClip } from "./recording-scene-custom";
import {
  croppedCustomPane,
  customCropWindow,
} from "./recording-scene-custom-crop";
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
        moving: next.operation === "cropMove" ? 0 : next.edges & 15,
        next: customCropWindow(pane.box, {
          anchor,
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
