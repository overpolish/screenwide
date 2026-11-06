// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import { cameraFramingOf, framingWithin } from "./recording-scene-framing";
import { SceneRect } from "./recording-scene-geometry";
import {
  SceneCamera,
  basePlacement,
  sceneBoxes,
} from "./recording-scene-placement";
import {
  RecordingSceneClip,
  SceneBox,
  WHOLE_FRAMING,
  sceneNeedsCamera,
} from "./recording-scenes";

/** The furthest a box may grow and shrink, as shares of the canvas. The twins
 * of the limits in `src-tauri/crates/scenes/src/model.rs`. */
const MAX_BOX_SHARE = 4;
export const MIN_BOX_SHARE = 0.02;

/** How far a select gesture may scale `box`: as far as keeps it between the
 * smallest and largest a box may be. */
const heldScale = (box: SceneBox, scale: number) => {
  const low = MIN_BOX_SHARE / Math.max(Math.min(box.width, box.height), 1e-9);
  const high = MAX_BOX_SHARE / Math.max(Math.max(box.width, box.height), 1e-9);
  return Math.min(high, Math.max(low, scale));
};

/**
 * The box a select gesture leaves: its corner moved `delta` shares of the
 * canvas and its size scaled by `scale`, its shape kept. It stays between the
 * smallest and largest a box may be, its middle on the canvas, so it can
 * always be found and dragged back. The twin of `SceneBox::moved`.
 */
export const movedSceneBox = (
  box: SceneBox,
  { delta, scale }: { delta: { x: number; y: number }; scale: number },
): SceneBox => {
  const held = heldScale(box, scale);
  const width = box.width * held;
  const height = box.height * held;
  return {
    height,
    width,
    x: Math.min(1 - width / 2, Math.max(-width / 2, box.x + delta.x)),
    y: Math.min(1 - height / 2, Math.max(-height / 2, box.y + delta.y)),
  };
};

/**
 * `movedSceneBox` by a gesture on the box padded outwards by `pad` shares of
 * the canvas on each side, where a scene's inset draws the handles. The
 * padding scales with the box, so the box's corner travels the padding's
 * growth further than the padded corner the gesture reports. The twin of
 * `SceneBox::moved_padded`.
 */
export const movedPaddedSceneBox = (
  box: SceneBox,
  {
    delta,
    pad,
    scale,
  }: {
    delta: { x: number; y: number };
    pad: { x: number; y: number };
    scale: number;
  },
): SceneBox => {
  const grown = heldScale(box, scale) - 1;
  return movedSceneBox(box, {
    delta: { x: delta.x + pad.x * grown, y: delta.y + pad.y * grown },
    scale,
  });
};

/**
 * `clip` made custom: its panes kept in the boxes it puts them in now, on the
 * screen's `output`, and showing what they show there, so the change alone
 * moves nothing. A full scene's camera, where `camera` gives one drawn into
 * the picture, gets the recording's own box. A clip already custom, or an
 * output with no placement to measure from, is left as it is.
 */
export function customRecordingSceneClip({
  camera,
  clip,
  output,
}: {
  camera: SceneCamera | null;
  clip: RecordingSceneClip;
  output: ScreenshotOutputSettings;
}): RecordingSceneClip {
  if (clip.boxes || output.cropWidth <= 0 || output.cropHeight <= 0)
    return clip;
  const canvas = { height: output.height, width: output.width };
  const base = basePlacement(output, camera);
  const boxes = sceneBoxes(clip, { base, camera, canvas });
  const cameraBox =
    boxes.camera ?? (camera && clip.preset === "full" ? base.frame : null);
  const share = (rect: SceneRect) =>
    movedSceneBox(
      {
        height: rect.height / Math.max(1, canvas.height),
        width: rect.width / Math.max(1, canvas.width),
        x: rect.x / Math.max(1, canvas.width),
        y: rect.y / Math.max(1, canvas.height),
      },
      { delta: { x: 0, y: 0 }, scale: 1 },
    );
  // An unset camera framing means a different picture once the camera has a
  // box of its own, so the one the clip shows now is written down.
  const cameraFraming =
    camera && cameraBox
      ? framingWithin(
          clip.camera ??
            (sceneNeedsCamera(clip)
              ? WHOLE_FRAMING
              : cameraFramingOf(camera.overlay, camera.aspect)),
          cameraBox,
          camera.aspect,
        )
      : clip.camera;
  return {
    ...clip,
    boxes: {
      screen: share(boxes.screen),
      ...(cameraBox ? { camera: share(cameraBox) } : {}),
    },
    ...(cameraFraming ? { camera: cameraFraming } : {}),
  };
}
