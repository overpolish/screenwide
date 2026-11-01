// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  boxesDrawing,
  presetDrawing,
  type SceneDrawing,
} from "../../glyphs/scene-layout-drawing";
import { CameraOverlaySettings } from "../../types";

import {
  recordingSceneLook,
  sceneRadius,
} from "./recording-scene-panel-snapshot";
import { SceneCamera } from "./recording-scene-placement";
import { RecordingSceneClip } from "./recording-scenes";

import type { SceneBoxes } from "../../../../bindings/SceneBoxes";
import type { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import type { SchematicComposition } from "../../tool-panels/scene/scene-schematic";

// A scene's layout as its drawings show it, read the way the Scene island
// reads the scene under the playhead: a custom scene by its own boxes, a
// full one by the recording's own, else by its preset, each on the video's
// shape with the screen turned as the scene turns it. The strip draws every
// scene so, and the island the one in hand, so a scene reads the same in
// both.

/** The whole video, where a scene shows no screen. */
export const WHOLE_FRAME = { height: 1, width: 1, x: 0, y: 0 };

/** A composition's panes as a custom layout's boxes. */
export const compositionBoxes = ({
  camera,
  cameraBehind,
  screen,
}: SchematicComposition): SceneBoxes => ({
  screen: screen ?? WHOLE_FRAME,
  ...(camera ? { camera } : {}),
  ...(cameraBehind ? { cameraBehind } : {}),
});

export function sceneClipDrawing(
  clip: RecordingSceneClip,
  source: {
    /** The camera a scene can place: one drawn into the picture. */
    camera: SceneCamera | null;
    cameraAspect: number;
    cameraOverlay: CameraOverlaySettings;
    output: ScreenshotOutputSettings;
  },
): SceneDrawing {
  const look = recordingSceneLook(source);
  const shape = {
    aspect: look.canvasAspect,
    cameraAspect: look.cameraAspect,
    cameraRadius: sceneRadius(clip, "camera", source),
    rotation: clip.rotation,
    screen: look.composition.screen,
    screenAspect: look.screenAspect,
  };
  if (clip.boxes) return boxesDrawing(clip.boxes, shape);
  if (clip.preset === "full")
    return boxesDrawing(compositionBoxes(look.composition), shape);
  return presetDrawing(clip.preset, clip.variant ?? {}, shape);
}
