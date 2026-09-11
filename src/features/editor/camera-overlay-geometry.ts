// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  screenshotLayout,
  ScreenshotOutputSettings,
} from "./screenshot-output";
import { CameraOverlaySettings, RecordingPreviewPane } from "./types";

type OverlayRect = {
  height: number;
  width: number;
  x: number;
  y: number;
};

type CameraModeBoundaryOptions = {
  cameraOutput: ScreenshotOutputSettings;
  cameraSource: { height: number; width: number };
  screenOutput: { height: number; width: number };
  settings: CameraOverlaySettings;
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

/**
 * Carry camera crop and corner radius from split-track output into the baked
 * overlay frame.
 *
 * The split preview stores the camera crop in the camera track's output
 * settings, while baked preview stores the same visible window as the
 * overlay's frame rectangle. Converting at the bake boundary keeps the crop
 * visually and semantically intact instead of resetting it to the full camera
 * image.
 */
export const cameraOverlayWithCameraCrop = ({
  cameraOutput,
  cameraSource,
  screenOutput,
  settings,
}: {
  cameraOutput: ScreenshotOutputSettings;
  cameraSource: { height: number; width: number };
  screenOutput: { height: number; width: number };
  settings: CameraOverlaySettings;
}): CameraOverlaySettings => {
  const screen = {
    height: screenOutput.height,
    kind: "screen" as const,
    sourceHeight: screenOutput.height,
    sourceWidth: screenOutput.width,
    width: screenOutput.width,
    x: 0,
    y: 0,
  };
  const camera = {
    height: cameraSource.height,
    kind: "camera" as const,
    sourceHeight: cameraSource.height,
    sourceWidth: cameraSource.width,
    width: cameraSource.width,
    x: 0,
    y: 0,
  };
  const geometry = cameraOverlayGeometry(screen, camera, settings);
  const layout = screenshotLayout(cameraSource, cameraOutput);
  const cropLeft =
    geometry.camera.x +
    ((layout.crop.x - layout.image.x) / Math.max(1, layout.image.width)) *
      geometry.camera.width;
  const cropTop =
    geometry.camera.y +
    ((layout.crop.y - layout.image.y) / Math.max(1, layout.image.height)) *
      geometry.camera.height;
  const cropWidth =
    (layout.crop.width / Math.max(1, layout.image.width)) *
    geometry.camera.width;
  const cropHeight =
    (layout.crop.height / Math.max(1, layout.image.height)) *
    geometry.camera.height;

  return {
    ...settings,
    frameHeight: cropHeight,
    frameWidth: cropWidth,
    frameX: cropLeft,
    frameY: cropTop,
    radiusPercent: cameraOutput.radiusPercent,
  };
};

/** Convert baked overlay crop and radius back into split camera settings. */
const cameraOutputWithOverlayCrop = ({
  cameraOutput,
  cameraSource,
  screenOutput,
  settings,
}: CameraModeBoundaryOptions): ScreenshotOutputSettings => {
  const screen = {
    height: screenOutput.height,
    kind: "screen" as const,
    sourceHeight: screenOutput.height,
    sourceWidth: screenOutput.width,
    width: screenOutput.width,
    x: 0,
    y: 0,
  };
  const camera = {
    height: cameraSource.height,
    kind: "camera" as const,
    sourceHeight: cameraSource.height,
    sourceWidth: cameraSource.width,
    width: cameraSource.width,
    x: 0,
    y: 0,
  };
  const geometry = cameraOverlayGeometry(screen, camera, settings);
  const layout = screenshotLayout(cameraSource, cameraOutput);
  const cropX =
    layout.image.x +
    ((geometry.frame.x - geometry.camera.x) /
      Math.max(1, geometry.camera.width)) *
      layout.image.width;
  const cropY =
    layout.image.y +
    ((geometry.frame.y - geometry.camera.y) /
      Math.max(1, geometry.camera.height)) *
      layout.image.height;
  const cropWidth =
    (geometry.frame.width / Math.max(1, geometry.camera.width)) *
    layout.image.width;
  const cropHeight =
    (geometry.frame.height / Math.max(1, geometry.camera.height)) *
    layout.image.height;
  return {
    ...cameraOutput,
    cropHeight,
    cropWidth,
    cropX,
    cropY,
    radiusPercent: settings.radiusPercent,
  };
};

/** Whether the overlay frame is acting as a crop window rather than the full image. */
const cameraOverlayHasCrop = ({
  cameraSource,
  screenOutput,
  settings,
}: {
  cameraSource: { height: number; width: number };
  screenOutput: { height: number; width: number };
  settings: CameraOverlaySettings;
}) => {
  const screen = {
    height: screenOutput.height,
    kind: "screen" as const,
    sourceHeight: screenOutput.height,
    sourceWidth: screenOutput.width,
    width: screenOutput.width,
    x: 0,
    y: 0,
  };
  const camera = {
    height: cameraSource.height,
    kind: "camera" as const,
    sourceHeight: cameraSource.height,
    sourceWidth: cameraSource.width,
    width: cameraSource.width,
    x: 0,
    y: 0,
  };
  const geometry = cameraOverlayGeometry(screen, camera, settings);
  const epsilon = 0.000_001;
  return (
    Math.abs(geometry.frame.x - geometry.camera.x) > epsilon ||
    Math.abs(geometry.frame.y - geometry.camera.y) > epsilon ||
    Math.abs(geometry.frame.width - geometry.camera.width) > epsilon ||
    Math.abs(geometry.frame.height - geometry.camera.height) > epsilon
  );
};

/** Preserve the baked camera's crop and radius when returning to split mode. */
export const cameraOutputWithCameraOverlay = (
  options: CameraModeBoundaryOptions,
): ScreenshotOutputSettings =>
  cameraOverlayHasCrop(options)
    ? cameraOutputWithOverlayCrop(options)
    : {
        ...options.cameraOutput,
        radiusPercent: options.settings.radiusPercent,
      };
