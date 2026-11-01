// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
} from "./screenshot/screenshot-output";

import type { CameraOverlaySettings } from "./types";

// Reset placement puts a layer back where a fresh project places it,
// keeping what it shows: its crop, padding, corners, shadow and turn. A
// fresh project fits the whole picture inside its box (the canvas, the
// camera's corner, a scene's pane) about the box's middle, so the reset fits
// what the layer shows now the same way. A layer never cropped lands
// exactly where a fresh project put it; a cropped one fills the box as the
// whole picture would. The picture behind the frame moves and scales with
// it, so the crop stays the same part of the picture.

export type PlacedRect = {
  height: number;
  width: number;
  x: number;
  y: number;
};

/** How far, in output pixels, a layer may sit from its default placement
 * and still be at it: a drag rounds to whole pixels and a tween settles
 * within a fraction of one. */
const PLACEMENT_TOLERANCE = 0.5;

/** `frame`'s shape fitted inside `box` about its middle. */
export const fittedRect = (
  frame: { height: number; width: number },
  box: PlacedRect,
): PlacedRect => {
  const scale = Math.min(
    box.width / Math.max(Number.EPSILON, frame.width),
    box.height / Math.max(Number.EPSILON, frame.height),
  );
  const width = frame.width * scale;
  const height = frame.height * scale;
  return {
    height,
    width,
    x: box.x + (box.width - width) / 2,
    y: box.y + (box.height - height) / 2,
  };
};

/** Whether `a` and `b` are the same rect to within `tolerance`. */
export const isSameRect = (
  a: PlacedRect,
  b: PlacedRect,
  tolerance = PLACEMENT_TOLERANCE,
) =>
  Math.abs(a.x - b.x) <= tolerance &&
  Math.abs(a.y - b.y) <= tolerance &&
  Math.abs(a.width - b.width) <= tolerance &&
  Math.abs(a.height - b.height) <= tolerance;

const layerFrame = (settings: ScreenshotOutputSettings): PlacedRect => ({
  height: settings.cropHeight,
  width: settings.cropWidth,
  x: settings.cropX,
  y: settings.cropY,
});

/** `settings`' layer at its default placement: its frame (the crop with its
 * padding) fitted inside the canvas, the picture behind it with it. A layer
 * from before placement was measured has no frame and is left as it is. */
export const layerAtDefaultPlacement = (
  settings: ScreenshotOutputSettings,
): ScreenshotOutputSettings => {
  if (settings.cropWidth <= 0 || settings.cropHeight <= 0) return settings;
  const canvas = screenshotOutputDimensions(settings);
  const frame = fittedRect(layerFrame(settings), { ...canvas, x: 0, y: 0 });
  const scale = frame.width / settings.cropWidth;
  return {
    ...settings,
    cropHeight: frame.height,
    cropWidth: frame.width,
    cropX: frame.x,
    cropY: frame.y,
    imageWidth: settings.imageWidth * scale,
    imageX: frame.x + (settings.imageX - settings.cropX) * scale,
    imageY: frame.y + (settings.imageY - settings.cropY) * scale,
  };
};

export const isLayerAtDefaultPlacement = (settings: ScreenshotOutputSettings) =>
  isSameRect(
    layerFrame(settings),
    layerFrame(layerAtDefaultPlacement(settings)),
  );

const overlayFrame = (overlay: CameraOverlaySettings): PlacedRect => ({
  height: overlay.frameHeight,
  width: overlay.frameWidth,
  x: overlay.frameX,
  y: overlay.frameY,
});

/** A camera drawn into the screen's picture at its default placement: its
 * window fitted inside the window of `fresh`, the overlay a fresh project
 * gives it on this canvas, the camera behind the window with it. */
export const overlayAtDefaultPlacement = (
  overlay: CameraOverlaySettings,
  fresh: CameraOverlaySettings,
): CameraOverlaySettings => {
  if (overlay.frameWidth <= 0 || overlay.frameHeight <= 0) return overlay;
  const frame = fittedRect(overlayFrame(overlay), overlayFrame(fresh));
  const scale = frame.width / overlay.frameWidth;
  return {
    ...overlay,
    cameraWidth: overlay.cameraWidth * scale,
    cameraX: frame.x + (overlay.cameraX - overlay.frameX) * scale,
    cameraY: frame.y + (overlay.cameraY - overlay.frameY) * scale,
    frameHeight: frame.height,
    frameWidth: frame.width,
    frameX: frame.x,
    frameY: frame.y,
  };
};

export const isOverlayAtDefaultPlacement = (
  overlay: CameraOverlaySettings,
  fresh: CameraOverlaySettings,
) =>
  isSameRect(
    overlayFrame(overlay),
    overlayFrame(overlayAtDefaultPlacement(overlay, fresh)),
  );
