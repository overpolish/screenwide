// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CameraOverlaySettings } from "../types";

/**
 * Resize the baked camera overlay about an anchor, the way the drag on it in
 * the preview does.
 *
 * The overlay is two rectangles in the screen output's own pixels: the camera
 * image, held by its centre and its width, and the crop window in front of it.
 * A resize scales both, so the crop keeps showing the same part of the camera
 * picture; `frame` is where the resized crop window's top left corner lands,
 * which is what decides the point the scale happens about.
 */
export const scaledCameraOverlay = (
  settings: CameraOverlaySettings,
  scale: number,
  frame: { x: number; y: number },
): CameraOverlaySettings => {
  // The camera image is carried by the same scale about the same anchor, so
  // it stays put behind the crop window rather than sliding out from under it.
  const anchored = (value: number, start: number, next: number) => {
    if (Math.abs(scale - 1) < 1e-9) return value;
    const anchor = (next - start * scale) / (1 - scale);
    return anchor + (value - anchor) * scale;
  };
  return {
    ...settings,
    cameraWidth: settings.cameraWidth * scale,
    cameraX: anchored(settings.cameraX, settings.frameX, frame.x),
    cameraY: anchored(settings.cameraY, settings.frameY, frame.y),
    frameHeight: settings.frameHeight * scale,
    frameWidth: settings.frameWidth * scale,
    frameX: frame.x,
    frameY: frame.y,
  };
};
