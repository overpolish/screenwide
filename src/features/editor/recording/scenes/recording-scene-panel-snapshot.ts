// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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

/** How round `clip` draws `pane`'s corners, in percent of its shorter side:
 * the recording's own radius where the scene does not round it. */
export const sceneRadius = (
  clip: RecordingSceneClip,
  pane: ScenePane,
  {
    cameraOverlay,
    output,
  }: { cameraOverlay: CameraOverlaySettings; output: ScreenshotOutputSettings },
) =>
  clip.radius?.[pane] ??
  (pane === "camera" ? cameraOverlay.radiusPercent : output.radiusPercent);

/**
 * What the scene islands are shown of the scene under the playhead,
 * `current`, on the screen's `output`: its layout and each pane's framing
 * and corners, where it shows that pane, with the selected track's pane on
 * its own.
 */
export function toolPanelSceneOf({
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
    sceneRadius(clip, pane, { cameraOverlay, output });
  const paneOf = (pane: ScenePane): ToolPanelScenePane | null => {
    const found = current
      ? scenePaneFraming({ camera, clip: current, output, pane })
      : null;
    return current && found
      ? {
          framing: found.framing,
          inset: pane === "screen" ? (current.inset ?? 0) : 0,
          isAtLayout: isScenePaneAtLayout(current, pane, { camera, output }),
          radius: radiusOf(current, pane),
        }
      : null;
  };
  return {
    boxes: current?.boxes
      ? {
          camera: current.boxes.camera ?? null,
          cameraBehind: current.boxes.cameraBehind ?? false,
          screen: current.boxes.screen,
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
    hasSceneAtPlayhead: current !== null,
    isBaked,
    panes: { camera: paneOf("camera"), screen: paneOf("screen") },
    preset: current?.preset ?? null,
    radius: current && currentPane ? radiusOf(current, currentPane.pane) : null,
    rotation: current ? (current.rotation ?? UPRIGHT_ROTATION) : null,
    sceneId: current?.id ?? null,
    screenAspect: look.screenAspect,
    variant: current ? (current.variant ?? {}) : null,
  };
}
