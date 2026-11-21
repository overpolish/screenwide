// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CropAspect } from "../../island/crop-aspect-choice";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import {
  shapedCustomPane,
  uncroppedCustomPane,
} from "./recording-scene-custom-crop";
import { scenePaneFraming } from "./recording-scene-pane-framing";
import { SceneCamera } from "./recording-scene-placement";
import { RecordingSceneClip, WHOLE_FRAMING } from "./recording-scenes";

// The crop is each scene's own: its screen's box and framing, cropped in
// place as a custom scene's is. A preset keeps its layout, laying the
// screen out at the crop's shape (`shapes`).

type Look = { camera: SceneCamera | null; output: ScreenshotOutputSettings };

const canvasOf = (output: ScreenshotOutputSettings) => ({
  height: output.height,
  width: output.width,
});

/** `clip` with its screen cropped to `aspect`. */
export function croppedSceneScreen(
  clip: RecordingSceneClip,
  aspect: CropAspect,
  { camera, output }: Look,
): RecordingSceneClip {
  const pane = scenePaneFraming({ camera, clip, output, pane: "screen" });
  const canvas = canvasOf(output);
  const shaped = pane ? shapedCustomPane(pane, { aspect, canvas }) : null;
  if (!shaped) return clip;
  if (clip.boxes)
    return {
      ...clip,
      boxes: { ...clip.boxes, screen: shaped.box },
      screen: shaped.framing,
    };
  const shape =
    (shaped.box.width * canvas.width) /
    Math.max(shaped.box.height * canvas.height, Number.EPSILON);
  return {
    ...clip,
    screen: shaped.framing,
    shapes: { ...clip.shapes, screen: shape },
  };
}

/** `clip` with its screen's crop taken off. A preset's screen goes back to
 * its picture's shape, its crop going with the zoom; a custom scene's box
 * grows back to the picture's shape. */
export function uncroppedSceneScreen(
  clip: RecordingSceneClip,
  { camera, output }: Look,
): RecordingSceneClip {
  if (!clip.boxes) {
    const { shapes, ...rest } = clip;
    const camerasOwn = shapes?.camera;
    return {
      ...rest,
      screen: WHOLE_FRAMING,
      ...(camerasOwn === undefined ? {} : { shapes: { camera: camerasOwn } }),
    };
  }
  const pane = scenePaneFraming({ camera, clip, output, pane: "screen" });
  if (!pane) return clip;
  const whole = uncroppedCustomPane(pane, { canvas: canvasOf(output) });
  return {
    ...clip,
    boxes: { ...clip.boxes, screen: whole.box },
    screen: whole.framing,
  };
}
