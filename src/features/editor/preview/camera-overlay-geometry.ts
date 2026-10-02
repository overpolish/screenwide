// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CameraOverlaySettings, RecordingPreviewPane } from "../types";

type OverlayRect = {
  height: number;
  width: number;
  x: number;
  y: number;
};

export type CameraOverlayGeometry = {
  camera: OverlayRect;
  frame: OverlayRect;
  radius: number;
};

/**
 * The overlay's two rectangles, in the screen output's own pixels.
 *
 * Both are stored in those pixels, so only the camera image's height has to be
 * worked out, and it follows the camera media's aspect: a crop that is valid
 * here reconstructs identically against the source files during export.
 */
export const cameraOverlayGeometry = (
  _screen: RecordingPreviewPane,
  camera: RecordingPreviewPane,
  settings: CameraOverlaySettings,
): CameraOverlayGeometry => {
  const frame = {
    height: settings.frameHeight,
    width: settings.frameWidth,
    x: settings.frameX,
    y: settings.frameY,
  };
  const cameraWidth = settings.cameraWidth;
  const cameraHeight =
    cameraWidth * (camera.sourceHeight / Math.max(1, camera.sourceWidth));
  return {
    camera: {
      height: cameraHeight,
      width: cameraWidth,
      x: settings.cameraX - cameraWidth / 2,
      y: settings.cameraY - cameraHeight / 2,
    },
    frame,
    radius:
      (Math.min(frame.width, frame.height) * settings.radiusPercent) / 100,
  };
};

/** Display-only overlay used by the native compositor while camera crop is active. */
export const uncroppedCameraPreviewOverlay = (
  screen: RecordingPreviewPane,
  camera: RecordingPreviewPane,
  settings: CameraOverlaySettings,
): CameraOverlaySettings => {
  const { camera: image } = cameraOverlayGeometry(screen, camera, settings);
  return {
    ...settings,
    frameHeight: image.height,
    frameWidth: image.width,
    frameX: image.x,
    frameY: image.y,
    radiusPercent: 0,
  };
};
