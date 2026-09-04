// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type CameraPreviewDimensions = {
  height: number;
  width: number;
};

/** Fits any camera resolution inside a 16:9 stage without cropping it. */
export const cameraPreviewFitClassName = ({
  height,
  width,
}: CameraPreviewDimensions) =>
  width * 9 >= height * 16
    ? "h-auto max-h-full w-full"
    : "h-full w-auto max-w-full";
