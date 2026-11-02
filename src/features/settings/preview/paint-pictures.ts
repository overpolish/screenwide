// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  pictureSize,
  sceneOf,
  type SettingsPreviewScene,
  showsScreenwide,
} from "./scenes/preview-scenes";
import {
  type Painter,
  type SceneDesktop,
  sceneFonts,
} from "./scenes/scene-kit";

import type { PaintedPictures } from "./settings-preview-api";

// The pictures a scene is drawn on, painted by the page in device pixels:
// the desktop, and for a scene that shows Screenwide's windows, the same
// desktop with them on it.

function paintOn(
  canvas: HTMLCanvasElement,
  scene: SettingsPreviewScene,
  { desktop, painters }: { desktop: SceneDesktop; painters: Painter[] },
) {
  const size = pictureSize(scene);
  const scale = window.devicePixelRatio;
  canvas.width = Math.round(size.width * scale);
  canvas.height = Math.round(size.height * scale);
  const context = canvas.getContext("2d");
  if (!context) return null;
  context.scale(scale, scale);
  for (const paint of painters) paint(context, size, desktop);
  return context;
}

/** Every picture the app draws `scene` from. */
export async function paintPictures(
  scene: SettingsPreviewScene,
  desktop: SceneDesktop,
): Promise<PaintedPictures | null> {
  await sceneFonts();
  const definition = sceneOf(scene);
  const layers =
    definition.kind === "recording" && definition.paintScreenwide
      ? [[definition.paint], [definition.paint, definition.paintScreenwide]]
      : [[definition.paint]];
  const canvas = document.createElement("canvas");
  const rgba: Uint8Array[] = [];
  for (const painters of layers) {
    const context = paintOn(canvas, scene, { desktop, painters });
    if (!context) return null;
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height);
    rgba.push(new Uint8Array(pixels.data.buffer));
  }
  return { height: canvas.height, rgba, width: canvas.width };
}

/** The picture shown where the app draws nothing, such as in Storybook:
 * the desktop as the scene leaves it, without the app's drawing. */
export async function paintStill(
  canvas: HTMLCanvasElement,
  scene: SettingsPreviewScene,
  desktop: SceneDesktop,
) {
  await sceneFonts();
  const definition = sceneOf(scene);
  const screenwide =
    definition.kind === "recording" && showsScreenwide(scene)
      ? definition.paintScreenwide
      : undefined;
  paintOn(canvas, scene, {
    desktop,
    painters: screenwide ? [definition.paint, screenwide] : [definition.paint],
  });
}
