// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CameraOverlaySettings } from "./types";

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

/**
 * Size the baked camera overlay to what a field asked for, keeping it where it
 * sits.
 *
 * The overlay is only ever scaled uniformly - the crop window and the camera
 * behind it together - so a size is read as a scale taken from whichever of
 * width or height was offered, and the frame keeps the centre it had.
 */
export const withCameraOverlaySize = (
  settings: CameraOverlaySettings,
  size: { height?: number; width?: number },
): CameraOverlaySettings => {
  const requested =
    size.width !== undefined && settings.frameWidth > 0
      ? size.width / settings.frameWidth
      : size.height !== undefined && settings.frameHeight > 0
        ? size.height / settings.frameHeight
        : 1;
  const scale = Number.isFinite(requested) && requested > 0 ? requested : 1;
  const centerX = settings.frameX + settings.frameWidth / 2;
  const centerY = settings.frameY + settings.frameHeight / 2;
  return scaledCameraOverlay(settings, scale, {
    x: centerX - (settings.frameWidth * scale) / 2,
    y: centerY - (settings.frameHeight * scale) / 2,
  });
};
