// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  SceneRect,
  recordingScenePanes,
} from "../../recording/scenes/recording-scene-geometry";
import { SceneVariant } from "../../recording/scenes/recording-scene-variant";
import { RecordingScenePreset } from "../../recording/scenes/recording-scenes";

/** A pane's place as shares of the canvas: `x` and `width` of its width,
 * `y` and `height` of its height. */
export type SchematicRect = SceneRect;

/** Where a scene puts the screen and the camera as shares of the canvas,
 * null for a pane it does not show. */
export type SchematicComposition = {
  camera: SchematicRect | null;
  screen: SchematicRect | null;
};

/**
 * Where `preset` puts the screen and the camera on a canvas of
 * `canvasAspect`, for a screen of `screenAspect` and a camera of
 * `cameraAspect`, each width over height, as `variant` has it, as shares of
 * the canvas: the frame the preview and the export draw, shrunk to a button.
 * A full scene draws the recording's own `composition`, and a screen-only
 * one its screen alone.
 */
export function sceneSchematic(
  preset: RecordingScenePreset,
  {
    cameraAspect,
    canvasAspect,
    composition,
    screenAspect,
    variant,
  }: {
    cameraAspect: number;
    canvasAspect: number;
    composition: SchematicComposition;
    screenAspect: number;
    variant?: SceneVariant;
  },
): SchematicComposition {
  if (preset === "full") return composition;
  if (preset === "screen-only")
    return { camera: null, screen: composition.screen };
  const { camera, screen } = recordingScenePanes(
    preset,
    { height: 1, width: canvasAspect },
    { cameraAspect, screenAspect, variant },
  );
  const share = (rect: SceneRect | null): SchematicRect | null =>
    rect && {
      height: rect.height,
      width: rect.width / canvasAspect,
      x: rect.x / canvasAspect,
      y: rect.y,
    };
  return { camera: share(camera), screen: share(screen) };
}
