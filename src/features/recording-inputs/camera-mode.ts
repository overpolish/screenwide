// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CameraDevice, CameraResolution } from "./types";

const isWidescreen = (mode: CameraResolution) =>
  Math.abs(mode.width / mode.height - 16 / 9) < 0.01;

const isLandscape = (mode: CameraResolution) => mode.width > mode.height;

/** The mode a camera records in: its largest widescreen picture, as a
 * recording is framed, then its largest landscape one, then its largest of
 * any shape, as some cameras, Apple's among them, offer square and 4:3
 * pictures larger than their widest. Of one size, the cadence asked for
 * first wins (`cameraRequestFps`). */
export const bestCameraMode = (
  camera: CameraDevice,
  requestFps: number[],
): CameraResolution | null => {
  const preference = (mode: CameraResolution) => {
    const index = requestFps.indexOf(mode.fps);
    return index < 0 ? requestFps.length : index;
  };
  const shapes = [
    camera.modes.filter(isWidescreen),
    camera.modes.filter(isLandscape),
    camera.modes,
  ];
  const modes = shapes.find((shape) => shape.length > 0) ?? [];
  return modes.reduce<CameraResolution | null>((best, mode) => {
    if (!best) return mode;
    const area = mode.width * mode.height;
    const bestArea = best.width * best.height;
    if (area !== bestArea) return area > bestArea ? mode : best;
    return preference(mode) < preference(best) ? mode : best;
  }, null);
};
