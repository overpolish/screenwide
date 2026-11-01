// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fittedRect, isSameRect } from "../../layer-placement";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import { SceneRect } from "./recording-scene-geometry";
import { ScenePane } from "./recording-scene-order";
import {
  SceneCamera,
  basePlacement,
  sceneBoxes,
} from "./recording-scene-placement";
import { RecordingSceneClip, SceneBox } from "./recording-scenes";

// Reset placement over a scene: a custom scene's pane goes back in the box
// the scene's preset gives it, as the layer outside every scene goes back
// where a fresh project places it (`layer-placement.ts`). Its box keeps its
// shape, fitted in the preset's box about its middle, so a pane cropped in
// place keeps its crop and its framing still fills it; a pane only moved and
// sized, whose box keeps the preset's shape, lands exactly in it. A preset
// scene's panes are in its boxes already.

type Look = { camera: SceneCamera | null; output: ScreenshotOutputSettings };

const canvasOf = (output: ScreenshotOutputSettings) => ({
  height: Math.max(1, output.height),
  width: Math.max(1, output.width),
});

/** The box `clip`'s preset gives `pane`, in output pixels, or null where
 * the preset places none of its own. A full scene's camera keeps the
 * recording's own box, as making it custom gives it. */
const layoutBox = (
  clip: RecordingSceneClip,
  pane: ScenePane,
  { camera, output }: Look,
): SceneRect | null => {
  const { boxes: _boxes, ...preset } = clip;
  const base = basePlacement(output, camera);
  const laid = sceneBoxes(preset, { base, camera, canvas: canvasOf(output) });
  if (pane === "screen") return laid.screen;
  return laid.camera ?? (camera && clip.preset === "full" ? base.frame : null);
};

/** `clip`'s custom box for `pane` and where Reset placement puts it, both
 * in output pixels; null where there is nothing to put back. */
const paneReset = (
  clip: RecordingSceneClip,
  pane: ScenePane,
  look: Look,
): { at: SceneRect; box: SceneRect } | null => {
  const share = clip.boxes?.[pane];
  const layout = share ? layoutBox(clip, pane, look) : null;
  if (!share || !layout) return null;
  const canvas = canvasOf(look.output);
  const box = {
    height: share.height * canvas.height,
    width: share.width * canvas.width,
    x: share.x * canvas.width,
    y: share.y * canvas.height,
  };
  return { at: fittedRect(box, layout), box };
};

/** Whether `clip`'s `pane` sits in the box its layout gives it, so Reset
 * placement has nothing to put back. */
export const isScenePaneAtLayout = (
  clip: RecordingSceneClip,
  pane: ScenePane,
  look: Look,
) => {
  const reset = paneReset(clip, pane, look);
  return reset === null || isSameRect(reset.box, reset.at);
};

/** `clip` with `pane` back in the box its layout gives it, its framing,
 * crop, corners and turn as they were. */
export const scenePaneAtLayout = (
  clip: RecordingSceneClip,
  pane: ScenePane,
  look: Look,
): RecordingSceneClip => {
  const reset = paneReset(clip, pane, look);
  if (!reset || !clip.boxes) return clip;
  const canvas = canvasOf(look.output);
  const box: SceneBox = {
    height: reset.at.height / canvas.height,
    width: reset.at.width / canvas.width,
    x: reset.at.x / canvas.width,
    y: reset.at.y / canvas.height,
  };
  return { ...clip, boxes: { ...clip.boxes, [pane]: box } };
};
