// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CropAspect } from "../../island/crop-aspect-choice";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import { customRecordingSceneClip } from "./recording-scene-custom";
import {
  shapedCustomPane,
  uncroppedCustomPane,
} from "./recording-scene-custom-crop";
import { scenePaneFraming } from "./recording-scene-pane-framing";
import { SceneCamera } from "./recording-scene-placement";
import { RecordingSceneClip, WHOLE_FRAMING } from "./recording-scenes";

// The crop is each scene's own: its screen's box and framing, cropped in
// place as a custom scene's is, a preset's made custom first, where the
// change alone moves nothing.

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
  const shaped = pane
    ? shapedCustomPane(pane, { aspect, canvas: canvasOf(output) })
    : null;
  const owned = shaped
    ? customRecordingSceneClip({ camera, clip, output })
    : clip;
  return shaped && owned.boxes
    ? {
        ...owned,
        boxes: { ...owned.boxes, screen: shaped.box },
        screen: shaped.framing,
      }
    : clip;
}

/** `clip` with its screen's crop taken off. A preset's box is the picture's
 * shape already, so its crop goes with the zoom; a custom scene's box grows
 * back to the picture's shape. */
export function uncroppedSceneScreen(
  clip: RecordingSceneClip,
  { camera, output }: Look,
): RecordingSceneClip {
  if (!clip.boxes) return { ...clip, screen: WHOLE_FRAMING };
  const pane = scenePaneFraming({ camera, clip, output, pane: "screen" });
  if (!pane) return clip;
  const whole = uncroppedCustomPane(pane, { canvas: canvasOf(output) });
  return {
    ...clip,
    boxes: { ...clip.boxes, screen: whole.box },
    screen: whole.framing,
  };
}
