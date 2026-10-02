// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import { cameraFramingOf, framingWithin } from "./recording-scene-framing";
import { SceneRect } from "./recording-scene-geometry";
import {
  SceneCamera,
  basePlacement,
  targeting,
} from "./recording-scene-placement";
import {
  RecordingSceneClip,
  SceneFraming,
  WHOLE_FRAMING,
  sceneNeedsCamera,
} from "./recording-scenes";

/** One pane of a scene as the Scene panel's zoom and position reach it. */
export type ScenePaneFraming = {
  aspect: number;
  /** Where the scene settles the pane, in output pixels. */
  box: SceneRect;
  framing: SceneFraming;
  pane: "camera" | "screen";
};

/**
 * The pane of `clip` the Scene panel's zoom and position reach, on the
 * screen's `output`: the camera while `selectsCamera` and the scene places
 * one drawn into the picture, else the screen. A scene that hides one pane
 * leaves only the other. Its framing is held where the picture still fills
 * the box the scene settles it in. Null for an output with no placement to
 * measure from.
 */
export function selectedPaneFraming({
  camera,
  clip,
  output,
  selectsCamera,
}: {
  camera: SceneCamera | null;
  clip: RecordingSceneClip;
  output: ScreenshotOutputSettings;
  selectsCamera: boolean;
}): ScenePaneFraming | null {
  if (output.cropWidth <= 0 || output.cropHeight <= 0) return null;
  const base = basePlacement(output, camera);
  const screenAspect = output.cropWidth / output.cropHeight;
  // A scene that stands idle for want of a camera still has a screen to zoom.
  const target =
    targeting({
      base,
      camera,
      canvas: { height: output.height, width: output.width },
    })(clip) ?? base;
  const reachesCamera =
    camera !== null &&
    target.opacity.camera > 0 &&
    (selectsCamera || target.opacity.screen === 0);
  if (reachesCamera) {
    const framing =
      clip.camera ??
      (sceneNeedsCamera(clip)
        ? WHOLE_FRAMING
        : cameraFramingOf(camera.overlay, camera.aspect));
    return {
      aspect: camera.aspect,
      box: target.frame,
      framing: framingWithin(framing, target.frame, camera.aspect),
      pane: "camera",
    };
  }
  return {
    aspect: screenAspect,
    box: target.screen,
    framing: framingWithin(
      clip.screen ?? WHOLE_FRAMING,
      target.screen,
      screenAspect,
    ),
    pane: "screen",
  };
}
