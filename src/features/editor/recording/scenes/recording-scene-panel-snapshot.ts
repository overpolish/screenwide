// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { canvasValue, Measure } from "../../layer-measure";
import { UPRIGHT_ROTATION } from "../../layer-rotation";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { CameraOverlaySettings } from "../../types";

import { drawnCameraFrame } from "./recording-scene-framing";
import { ScenePane } from "./recording-scene-order";
import {
  ScenePaneFraming,
  scenePaneFraming,
} from "./recording-scene-pane-framing";
import { isScenePaneAtLayout } from "./recording-scene-pane-reset";
import { scenePaneIsShown } from "./recording-scene-pane-shown";
import { SceneCamera } from "./recording-scene-placement";
import { RecordingSceneClip } from "./recording-scenes";

import type {
  ToolPanelScene,
  ToolPanelScenePane,
} from "../../tools/tool-panel-scene";

/** What the scenes are laid out on: the recording's own composition on the
 * screen's `output`, as shares of the canvas, the canvas's shape and the
 * screen's and camera's, each width over height. */
export function recordingSceneLook({
  camera,
  cameraAspect,
  cameraOverlay,
  output,
}: {
  /** The camera a scene can place: one drawn into the picture. */
  camera: SceneCamera | null;
  cameraAspect: number;
  cameraOverlay: CameraOverlaySettings;
  output: ScreenshotOutputSettings;
}) {
  // A composition from before placement was measured in output pixels has no
  // crop; the whole frame is its screen.
  const crop =
    output.cropWidth > 0 && output.cropHeight > 0
      ? {
          height: output.cropHeight,
          width: output.cropWidth,
          x: output.cropX,
          y: output.cropY,
        }
      : { height: output.height, width: output.width, x: 0, y: 0 };
  const share = (rect: typeof crop) => ({
    height: rect.height / Math.max(1, output.height),
    width: rect.width / Math.max(1, output.width),
    x: rect.x / Math.max(1, output.width),
    y: rect.y / Math.max(1, output.height),
  });
  return {
    cameraAspect,
    canvasAspect: output.width / Math.max(1, output.height),
    composition: {
      // Where the camera is drawn, which can differ from its stored box.
      camera: camera
        ? share(drawnCameraFrame(cameraOverlay, camera.aspect))
        : null,
      screen: share(crop),
    },
    screenAspect: crop.width / crop.height,
  };
}

/** How round `clip` draws the camera's corners: the recording's own where
 * the scene does not round them. */
export const sceneCameraRadius = (
  clip: RecordingSceneClip,
  cameraOverlay: CameraOverlaySettings,
): Measure => clip.radius?.camera ?? cameraOverlay.radius;

/** How round `clip` draws the screen's corners: the recording's own where
 * the scene does not round them. */
export const sceneScreenRadius = (
  clip: RecordingSceneClip,
  output: ScreenshotOutputSettings,
): Measure => clip.radius?.screen ?? output.radius;

/**
 * What the scene islands are shown of the scene in hand, `current` (the one
 * chosen on the strip, else the one under the playhead, `atPlayhead`), on
 * the screen's `output`: its layout and each pane's framing and corners,
 * where it shows that pane, with the selected track's pane on its own.
 */
export function toolPanelSceneOf({
  atPlayhead,
  camera,
  cameraAspect,
  cameraOverlay,
  clips,
  current,
  currentPane,
  hasCamera,
  isBaked,
  output,
}: {
  /** The scene under the playhead, null between scenes. */
  atPlayhead: RecordingSceneClip | null;
  /** The camera a scene can place: one drawn into the picture. */
  camera: SceneCamera | null;
  cameraAspect: number;
  cameraOverlay: CameraOverlaySettings;
  clips: RecordingSceneClip[];
  current: RecordingSceneClip | null;
  /** The pane of `current` the selected track is. */
  currentPane: ScenePaneFraming | null;
  hasCamera: boolean;
  isBaked: boolean;
  output: ScreenshotOutputSettings;
}): ToolPanelScene {
  const look = recordingSceneLook({
    camera,
    cameraAspect,
    cameraOverlay,
    output,
  });
  const radiusOf = (clip: RecordingSceneClip, pane: ScenePane) =>
    pane === "camera"
      ? sceneCameraRadius(clip, cameraOverlay)
      : sceneScreenRadius(clip, output);
  const paneOf = (pane: ScenePane): ToolPanelScenePane | null => {
    const found = current
      ? scenePaneFraming({ camera, clip: current, output, pane })
      : null;
    return current && found
      ? {
          framing: found.framing,
          // -1 is a padding no look gives.
          inset:
            pane === "screen" && current.inset
              ? (canvasValue(current.inset) ?? -1)
              : 0,
          isAtLayout: isScenePaneAtLayout(current, pane, { camera, output }),
          radius: radiusOf(current, pane),
        }
      : null;
  };
  return {
    background: current?.background ?? null,
    boxes: current?.boxes
      ? {
          camera: current.boxes.camera ?? null,
          cameraBehind: current.boxes.cameraBehind ?? false,
          cameraHidden: current.boxes.cameraHidden ?? false,
          screen: current.boxes.screen,
          screenHidden: current.boxes.screenHidden ?? false,
        }
      : null,
    cameraAspect,
    canvasAspect: look.canvasAspect,
    composition: look.composition,
    cursor: current?.cursor ?? null,
    followsFace: current?.followFace ?? cameraOverlay.followFace ?? false,
    framing: currentPane?.framing ?? null,
    framingPane: currentPane?.pane ?? "screen",
    hasAutoZooms: clips.some((clip) => clip.auto),
    hasCamera,
    hasSceneInHand: current !== null,
    hasScenes: clips.length > 0,
    isAtPlayhead: current !== null && current.id === atPlayhead?.id,
    isBaked,
    panes: { camera: paneOf("camera"), screen: paneOf("screen") },
    panesShown: current
      ? {
          camera: scenePaneIsShown(current, "camera"),
          screen: scenePaneIsShown(current, "screen"),
        }
      : null,
    preset: current?.preset ?? null,
    radius: current && currentPane ? radiusOf(current, currentPane.pane) : null,
    rotation: current ? (current.rotation ?? UPRIGHT_ROTATION) : null,
    sceneId: current?.id ?? null,
    screenAspect: look.screenAspect,
    variant: current ? (current.variant ?? {}) : null,
  };
}
