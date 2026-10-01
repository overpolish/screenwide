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

/**
 * The three rectangles a layer draws, in output pixels.
 *
 * Crop and image are stored in those pixels already, so the only thing left to
 * work out is the image's height, which always follows the source's aspect,
 * and where the source crop falls inside the image.
 */
export const screenshotLayout = (
  source: { height: number; width: number },
  settings: ScreenshotOutputSettings,
): ScreenshotLayout => {
  const imageWidth = Math.max(1, settings.imageWidth);
  const imageHeight = imageWidth * (source.height / Math.max(1, source.width));
  const image = {
    height: imageHeight,
    width: imageWidth,
    x: settings.imageX,
    y: settings.imageY,
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
      height: settings.cropHeight,
      width: settings.cropWidth,
      x: settings.cropX,
      y: settings.cropY,
    },
    image,
    sourceCrop,
  };
};
