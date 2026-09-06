// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { screenshotSourceCrop } from "./screenshot-output-settings";

import type { ScreenshotOutputSettings } from "./screenshot-output";

type Rect = { height: number; width: number; x: number; y: number };

export type ScreenshotLayout = {
  crop: Rect;
  image: Rect;
  sourceCrop: Rect;
};

export const screenshotLayout = (
  source: { height: number; width: number },
  output: { height: number; width: number },
  settings: ScreenshotOutputSettings,
): ScreenshotLayout => {
  const imageWidth =
    (output.width * Math.max(1, settings.screenshotImageWidthPercent)) / 100;
  const imageHeight = imageWidth * (source.height / Math.max(1, source.width));
  const image = {
    height: imageHeight,
    width: imageWidth,
    x: (output.width * settings.screenshotImageXPercent) / 100 - imageWidth / 2,
    y:
      (output.height * settings.screenshotImageYPercent) / 100 -
      imageHeight / 2,
  };
  const cropSource = screenshotSourceCrop(settings);
  const sourceCrop = {
    height: image.height * cropSource.height,
    width: image.width * cropSource.width,
    x: image.x + image.width * cropSource.x,
    y: image.y + image.height * cropSource.y,
  };
  return {
    crop: {
      height: (output.height * settings.screenshotCropHeightPercent) / 100,
      width: (output.width * settings.screenshotCropWidthPercent) / 100,
      x: (output.width * settings.screenshotCropXPercent) / 100,
      y: (output.height * settings.screenshotCropYPercent) / 100,
    },
    image,
    sourceCrop,
  };
};
