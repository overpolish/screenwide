// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTurned } from "../layer-rotation";
import { recordingScenePanes } from "../recording/scenes/recording-scene-geometry";

import type { RecordingScenePreset } from "../../../bindings/RecordingScenePreset";
import type { SceneBoxes } from "../../../bindings/SceneBoxes";
import type { SceneRotation } from "../../../bindings/SceneRotation";
import type { SceneVariant } from "../../../bindings/SceneVariant";

// Where a layout puts the panes on the video, as shares of it, so a drawing
// of it has the video's own shape: wide, square or tall. A preset is placed
// by the same rules the preview and the export use; a custom or saved
// layout by its own boxes. A scene's turn of the screen is drawn as the
// picture draws it, whatever the layout.

type Box = SceneBoxes["screen"];

/** The video's shape and its sources', each width over height, where the
 * recording itself puts the screen, as shares of the video, how round the
 * camera is, in percent of its shorter side, and how the scene turns the
 * screen, which every layout of it keeps. */
export type SceneShape = {
  aspect: number;
  cameraAspect: number;
  cameraRadius: number;
  screen: Box;
  screenAspect: number;
  rotation?: SceneRotation;
};

/** A layout on a video `aspect` wide to its height: the panes as shares of
 * it, the camera rounded by `radius` percent of its shorter side, and the
 * screen turned about its middle by `rotation`. */
export type SceneDrawing = {
  aspect: number;
  camera?: Box & { radius: number };
  rotation?: SceneRotation;
  screen?: Box;
};

/** The shape's turn, where it turns the screen at all. */
const turnOf = ({ rotation }: SceneShape) =>
  isTurned(rotation) ? { rotation } : {};

export type ScenePreset = Exclude<RecordingScenePreset, "full">;

export function presetDrawing(
  preset: ScenePreset,
  variant: SceneVariant,
  shape: SceneShape,
): SceneDrawing {
  const { aspect } = shape;
  // Screen only keeps the recording's own place for the screen.
  if (preset === "screen-only")
    return { aspect, screen: shape.screen, ...turnOf(shape) };
  const { camera, screen } = recordingScenePanes(
    preset,
    { height: 1, width: aspect },
    {
      cameraAspect: shape.cameraAspect,
      screenAspect: shape.screenAspect,
      variant,
    },
  );
  const share = (box: Box): Box => ({
    height: box.height,
    width: box.width / aspect,
    x: box.x / aspect,
    y: box.y,
  });
  return {
    aspect,
    camera: camera
      ? {
          ...share(camera),
          // A camera filling the video is cut by the video's own corners.
          radius: preset === "camera-only" ? 0 : shape.cameraRadius,
        }
      : undefined,
    screen: screen ? share(screen) : undefined,
    ...(screen ? turnOf(shape) : {}),
  };
}

/** A custom or saved layout, drawn from its boxes, which are shares of the
 * video. */
export function boxesDrawing(
  boxes: SceneBoxes,
  shape: SceneShape,
): SceneDrawing {
  return {
    aspect: shape.aspect,
    camera: boxes.camera
      ? { ...boxes.camera, radius: shape.cameraRadius }
      : undefined,
    screen: boxes.screen,
    ...turnOf(shape),
  };
}
