// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import {
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
} from "../../screenshot/screenshot-output";

import { customRecordingSceneClip } from "./recording-scene-custom";
import {
  croppedCustomPane,
  customCropWindow,
} from "./recording-scene-custom-crop";
import {
  boxEdit,
  framingEdit,
  keepingFaceSpot,
  PaneName,
  ScenePane,
} from "./recording-scene-gesture-edits";
import { SceneCamera } from "./recording-scene-placement";
import { RecordingSceneClip, rotatedSceneClip } from "./recording-scenes";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";

/** The scene at the playhead, while a gesture on its panes can reframe it:
 * every clip, the one under the playhead, and its panes. */
export type RecordingSceneTarget = {
  /** Null where the scene places no camera. */
  camera: ScenePane | null;
  clipId: string;
  clips: RecordingSceneClip[];
  screen: ScenePane;
};

type ActiveGesture =
  | { kind: "screen" }
  | {
      clipId: string;
      clips: RecordingSceneClip[];
      /** Each event's edit of the clip, from the clip as the gesture found
       * it; null leaves it as it was. */
      edit: (
        clip: RecordingSceneClip,
        event: RecordingSelectionGestureEvent,
      ) => RecordingSceneClip | null;
      kind: "edit";
    };

/**
 * Takes the gestures on the screen and the camera while the playhead is
 * inside a scene. A custom scene's own boxes move and resize with the select
 * tool, and the crop tool crops them in place, freely, the picture staying
 * where it sits. A preset's screen moves and resizes too, which first makes
 * the scene custom where it stands, as the panel's Custom does; only camera
 * only, which hides the screen, has none to move. The screen's crop is each
 * scene's own, so the crop tool crops any scene's screen in place too, a
 * preset's made custom by the crop's first travel. A preset's camera is
 * fixed where the scene puts it: with the select tool it pans its picture
 * inside its box or zooms it, and with the crop tool a window of the
 * camera's box's shape laid over its unzoomed picture picks the part the box
 * shows. Each edits the clip under the playhead; `scene_pane_gesture.rs` is
 * the native twin. The canvas's frame is left to the ordinary selection
 * gesture, as is a camera a custom scene has no box for and anything else
 * `applyGesture` answers it did not take.
 */
export function useRecordingSceneGesture({
  camera,
  editGesture,
  onClipsChange,
  primaryOutput,
  target,
}: {
  /** The camera a scene can place: one drawn into the picture. */
  camera: SceneCamera | null;
  editGesture: { beginGesture: () => void; endGesture: () => void };
  onClipsChange: (clips: RecordingSceneClip[]) => void;
  primaryOutput: ScreenshotOutputSettings;
  /** Null outside every scene. */
  target: RecordingSceneTarget | null;
}) {
  const activeRef = useRef<ActiveGesture | null>(null);
  const begin = (
    event: RecordingSelectionGestureEvent,
  ): ActiveGesture | null => {
    if (!target || (event.paneIndex !== 0 && event.paneIndex !== 1))
      return null;
    const paneName: PaneName = event.paneIndex === 1 ? "camera" : "screen";
    const pane = target[paneName];
    if (!pane) return null;
    const { clipId, clips } = target;
    // The rotation gizmo turns the screen, the one pane a scene turns,
    // reporting the turn it reaches: about X and Y as its travel, about Z as
    // its scale.
    if (event.operation === "rotate")
      return paneName === "screen"
        ? {
            clipId,
            clips,
            edit: (clip, next) =>
              rotatedSceneClip(clip, {
                x: next.deltaX,
                y: next.deltaY,
                z: next.scale,
              }),
            kind: "edit",
          }
        : null;
    const found = clips.find((clip) => clip.id === clipId);
    const { operation } = event;
    const isCrop = operation === "cropMove" || operation === "cropResize";
    const crops = isCrop || operation === "cropDraw";
    const owned =
      found &&
      !found.boxes &&
      paneName === "screen" &&
      (crops ||
        (found.preset !== "camera-only" &&
          (operation === "move" || operation === "resize")))
        ? customRecordingSceneClip({
            camera,
            clip: found,
            output: primaryOutput,
          })
        : found;
    const boxes = owned?.boxes;
    // The native gesture reports its travel as shares of the canvas.
    const canvas = screenshotOutputDimensions(primaryOutput);
    // A crop of a camera that follows the face keeps where the face sits.
    const follows =
      paneName === "camera" &&
      (found?.followFace ?? camera?.overlay.followFace ?? false);
    const keptSpot = (framing: ScenePane["framing"]) =>
      follows ? keepingFaceSpot(framing, pane.framing) : framing;
    // A scene's own boxes are cropped in place, drawn afresh included.
    if (owned && boxes?.[paneName] && crops) {
      const anchor = {
        x: pane.box.x + event.deltaX * canvas.width,
        y: pane.box.y + event.deltaY * canvas.height,
      };
      return {
        clipId,
        clips,
        edit: (_clip, next) => {
          if (
            next.operation !== "cropDraw" &&
            next.operation !== "cropMove" &&
            next.operation !== "cropResize"
          )
            return null;
          // A press that takes a preset's crop without moving it leaves the
          // scene as it was rather than making it custom.
          if (
            owned !== found &&
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
            slides: next.operation === "cropMove",
          });
          return {
            ...owned,
            boxes: { ...boxes, [paneName]: cropped.box },
            [paneName]: keptSpot(cropped.framing),
          };
        },
        kind: "edit",
      };
    }
    // With the camera a pane of its own, no scene places it, so its gestures
    // stay its own.
    if (!isCrop && operation !== "move" && operation !== "resize") return null;
    if (!isCrop && boxes) {
      if (!boxes[paneName]) return null;
      const edit = boxEdit(boxes, paneName, canvas);
      return {
        clipId,
        clips,
        // A press that selects a preset's screen without moving it leaves
        // the scene as it was rather than making it custom.
        edit: (_clip, next) =>
          owned !== found &&
          next.deltaX === 0 &&
          next.deltaY === 0 &&
          (next.operation !== "resize" || next.scale === 1)
            ? null
            : edit(owned, next),
        kind: "edit",
      };
    }
    if (!isCrop && paneName === "screen") return { kind: "screen" };
    return {
      clipId,
      clips,
      edit: (clip, next) => {
        const framing = framingEdit(pane, {
          delta: {
            x: next.deltaX * canvas.width,
            y: next.deltaY * canvas.height,
          },
          event: next,
        });
        return { ...clip, [paneName]: isCrop ? keptSpot(framing) : framing };
      },
      kind: "edit",
    };
  };
  const applyGesture = (event: RecordingSelectionGestureEvent) => {
    if (event.phase === "begin") {
      activeRef.current = begin(event);
      if (activeRef.current?.kind === "edit") editGesture.beginGesture();
      return activeRef.current !== null;
    }
    const active = activeRef.current;
    if (!active) return false;
    if (event.phase === "end" || event.phase === "cancel")
      activeRef.current = null;
    if (active.kind === "screen") return true;
    onClipsChange(
      active.clips.map((clip) =>
        clip.id === active.clipId && event.phase !== "cancel"
          ? (active.edit(clip, event) ?? clip)
          : clip,
      ),
    );
    // Keep the history gesture open through React's commit, as the ordinary
    // gesture does, so a late update cannot become a second undo entry.
    if (event.phase === "end" || event.phase === "cancel")
      requestAnimationFrame(editGesture.endGesture);
    return true;
  };
  return {
    applyGesture,
    gestureAccepted: () => activeRef.current !== null,
  };
}
