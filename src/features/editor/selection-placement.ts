// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { screenshotLayout } from "./screenshot-layout";
import {
  screenshotOutputDimensions,
  ScreenshotOutputSettings,
} from "./screenshot-output";

/**
 * Where the selected layer sits in the finished picture, in output pixels.
 *
 * The output stores placement as percentages of the canvas, and it stores two
 * rectangles: the visible crop by its top left corner, and the image behind it
 * by its centre. A panel shows one rectangle by its top left, so the crop is
 * what it reads and writes, and the image travels with it.
 */
export type SelectionPlacement = {
  height: number;
  width: number;
  x: number;
  y: number;
};

/** What a field may ask to change; the rest of the placement stays put. */
export type SelectionPlacementPatch = Partial<SelectionPlacement>;

const whole = (value: number) => Math.round(value);

export const selectionPlacement = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
): SelectionPlacement => {
  const output = screenshotOutputDimensions(settings);
  const { crop } = screenshotLayout(source, output, settings);
  return {
    height: whole(crop.height),
    width: whole(crop.width),
    x: whole(crop.x),
    y: whole(crop.y),
  };
};

/**
 * Place the selection at the asked-for size and position.
 *
 * A layer's image is only ever scaled uniformly - its height follows its
 * width and the source's aspect - so a size change is read as a scale about
 * the selection's top left corner, taken from whichever of width or height
 * the field offered. The move that follows carries crop and image together, so
 * an edit to X or Y never disturbs the crop the user set.
 */
export const withSelectionPlacement = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
  patch: SelectionPlacementPatch,
): ScreenshotOutputSettings => {
  const output = screenshotOutputDimensions(settings);
  const { crop, image } = screenshotLayout(source, output, settings);
  const requestedScale =
    patch.width !== undefined && crop.width > 0
      ? patch.width / crop.width
      : patch.height !== undefined && crop.height > 0
        ? patch.height / crop.height
        : 1;
  const scale =
    Number.isFinite(requestedScale) && requestedScale > 0 ? requestedScale : 1;
  const x = patch.x ?? crop.x;
  const y = patch.y ?? crop.y;
  const width = crop.width * scale;
  const height = crop.height * scale;
  const imageWidth = image.width * scale;
  const imageHeight = image.height * scale;
  const imageX = x + (image.x - crop.x) * scale;
  const imageY = y + (image.y - crop.y) * scale;
  return {
    ...settings,
    screenshotCropHeightPercent: (height * 100) / output.height,
    screenshotCropWidthPercent: (width * 100) / output.width,
    screenshotCropXPercent: (x * 100) / output.width,
    screenshotCropYPercent: (y * 100) / output.height,
    screenshotImageWidthPercent: (imageWidth * 100) / output.width,
    screenshotImageXPercent: ((imageX + imageWidth / 2) * 100) / output.width,
    screenshotImageYPercent: ((imageY + imageHeight / 2) * 100) / output.height,
  };
};
