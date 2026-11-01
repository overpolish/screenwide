// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";

import type { CameraOverlaySettings } from "../types";

// A canvas given a new size by the Frame island's dimensions takes what is
// in it along, so the composition keeps its proportions rather than a
// smaller canvas cutting into it. Each layer's middle keeps its place as a
// share of the canvas, and the layer is scaled by the lesser of the two
// axes' scales: it keeps its own shape, and a canvas narrowed in one axis
// alone still holds it. Its picture, padding and annotations, which are
// measured on the layer, scale with it.

/** How a canvas changing size moves and scales what is in it: each axis's
 * own scale for where a middle sits, and the lesser for how big it is. */
export type CanvasScale = { size: number; x: number; y: number };

/** The scale a canvas `from` big takes to be `to` big. */
export const canvasScale = (
  from: { height: number; width: number },
  to: { height: number; width: number },
): CanvasScale => {
  const x = to.width / Math.max(1, from.width);
  const y = to.height / Math.max(1, from.height);
  return { size: Math.min(x, y), x, y };
};

/** A canvas size as the output keeps it, in whole pixels. */
const canvasSize = (size: { height: number; width: number }) => ({
  height: Math.max(1, Math.round(size.height)),
  width: Math.max(1, Math.round(size.width)),
});

/** `settings` on a canvas of `size`, its layer scaled about its frame's
 * middle, which keeps its place as a share of the canvas. */
const scaledLayer = (
  settings: ScreenshotOutputSettings,
  scale: CanvasScale,
  size: { height: number; width: number },
): ScreenshotOutputSettings => {
  const middle = {
    x: settings.cropX + settings.cropWidth / 2,
    y: settings.cropY + settings.cropHeight / 2,
  };
  const moved = { x: middle.x * scale.x, y: middle.y * scale.y };
  const cropWidth = settings.cropWidth * scale.size;
  const cropHeight = settings.cropHeight * scale.size;
  return {
    ...settings,
    cropHeight,
    cropWidth,
    cropX: moved.x - cropWidth / 2,
    cropY: moved.y - cropHeight / 2,
    height: size.height,
    imageWidth: settings.imageWidth * scale.size,
    imageX: moved.x + (settings.imageX - middle.x) * scale.size,
    imageY: moved.y + (settings.imageY - middle.y) * scale.size,
    width: size.width,
  };
};

/** One output's canvas resized to `width` by `height`, its layer scaled
 * with it. */
export const scaleScreenshotCanvas = ({
  height,
  settings,
  width,
}: {
  height: number;
  settings: ScreenshotOutputSettings;
  width: number;
}): ScreenshotOutputSettings => {
  const size = canvasSize({ height, width });
  return scaledLayer(settings, canvasScale(settings, size), size);
};

/** A workspace's canvas resized to `width` by `height`, every layer scaled
 * with it. */
export const scaleScreenshotWorkspaceCanvas = ({
  height,
  settings,
  width,
}: {
  height: number;
  settings: ScreenshotWorkspaceOutputSettings;
  width: number;
}): ScreenshotWorkspaceOutputSettings => {
  const size = canvasSize({ height, width });
  const scale = canvasScale(settings, size);
  return {
    ...scaledLayer(settings, scale, size),
    items: settings.items.map((item) => ({
      ...item,
      output: scaledLayer(item.output, scale, size),
    })),
  };
};

/** A camera drawn into the screen's canvas, scaled with it as a layer is:
 * its window about its middle, the camera behind the window with it. */
export const scaledCameraOverlay = (
  overlay: CameraOverlaySettings,
  scale: CanvasScale,
): CameraOverlaySettings => {
  const middle = {
    x: overlay.frameX + overlay.frameWidth / 2,
    y: overlay.frameY + overlay.frameHeight / 2,
  };
  const moved = { x: middle.x * scale.x, y: middle.y * scale.y };
  const frameWidth = overlay.frameWidth * scale.size;
  const frameHeight = overlay.frameHeight * scale.size;
  return {
    ...overlay,
    cameraWidth: overlay.cameraWidth * scale.size,
    cameraX: moved.x + (overlay.cameraX - middle.x) * scale.size,
    cameraY: moved.y + (overlay.cameraY - middle.y) * scale.size,
    frameHeight,
    frameWidth,
    frameX: moved.x - frameWidth / 2,
    frameY: moved.y - frameHeight / 2,
  };
};
