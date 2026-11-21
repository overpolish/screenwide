// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import {
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
} from "../../screenshot/screenshot-output";

import {
  SceneEditGesture,
  sceneCropGesture,
} from "./recording-scene-crop-gesture";
import { customRecordingSceneClip } from "./recording-scene-custom";
import {
  boxEdit,
  framingEdit,
  keepingFaceSpot,
  PaneName,
  ScenePane,
} from "./recording-scene-gesture-edits";
import { SceneCamera } from "./recording-scene-placement";
import {
  RecordingSceneClip,
  rotatedSceneClip,
  sceneNeedsCamera,
} from "./recording-scenes";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";

/** The scene at the playhead, while a gesture on its panes can reframe it:
 * every clip, the one under the playhead, and its panes. */
export type RecordingSceneTarget = {
  /** Null where the scene places no camera. */
  camera: ScenePane | null;
  clipId: string;
  clips: RecordingSceneClip[];
  screen: ScenePane;
  /** How far the scene pads the screen's box as the paused frame draws it,
   * in canvas pixels on every side. */
  screenInset: number;
};

type ActiveGesture = { kind: "screen" } | SceneEditGesture;

/**
 * Takes the gestures on the screen and the camera while the playhead is
 * inside a scene, one rule for both panes: the select tool moves and resizes
 * a pane's box, and the crop tool picks what the pane shows. A custom
 * scene's own boxes move and resize, and are cropped in place, freely, the
 * picture staying where it sits. A preset's panes move and resize too,
 * which first makes the scene custom where it stands, as the panel's Custom
 * does (user asked, 2026-11-21); blank has no pane to move. The crop tool
 * crops a preset's panes in place too, keeping the preset: the pane is laid
 * out at the crop's shape once the crop tool leaves the scene. Each edits
 * the clip under the playhead; `scene_pane_gesture.rs` is the native twin.
 * The canvas's frame is left to the ordinary selection gesture, as is a
 * camera a custom scene has no box for and anything else `applyGesture`
 * answers it did not take.
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
    const placesBox = operation === "move" || operation === "resize";
    const frees =
      paneName === "screen"
        ? found?.preset !== "camera-only" && placesBox
        : placesBox &&
          found !== undefined &&
          (found.preset === "full" || sceneNeedsCamera(found));
    const owned =
      found && !found.boxes && frees
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
    if (owned && crops && (boxes?.[paneName] || !boxes))
      return sceneCropGesture({
        camera,
        canvas,
        clips,
        event,
        keptSpot,
        output: primaryOutput,
        pane,
        paneName,
        scene: owned,
      });
    // With the camera a pane of its own, no scene places it, so its gestures
    // stay its own.
    if (!isCrop && operation !== "move" && operation !== "resize") return null;
    if (!isCrop && boxes) {
      if (!boxes[paneName]) return null;
      const edit = boxEdit(boxes, paneName, {
        canvas,
        inset: paneName === "screen" ? target.screenInset : 0,
      });
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
    // Select moves a pane's box, above; what is left is a crop of what a
    // preset's camera shows, or a screen the scene keeps hidden.
    if (!isCrop) return paneName === "screen" ? { kind: "screen" } : null;
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
        return { ...clip, [paneName]: keptSpot(framing) };
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
    if (event.phase === "cancel") active.onCancel?.();
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
