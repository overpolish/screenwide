// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { screenshotLayout } from "./screenshot-layout";
import { ScreenshotOutputSettings } from "./screenshot-output";

/**
 * Where the selected layer sits in the finished picture, in output pixels.
 *
 * The output stores placement in those same pixels, and it stores two
 * rectangles: the visible crop and the image behind it, both by their top left
 * corner. A panel shows one rectangle, so the crop is what it reads and
 * writes, and the image travels with it.
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
): SelectionPlacement => ({
  height: whole(settings.cropHeight),
  width: whole(settings.cropWidth),
  x: whole(settings.cropX),
  y: whole(settings.cropY),
});

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
  const { crop, image } = screenshotLayout(source, settings);
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
  return {
    ...settings,
    cropHeight: crop.height * scale,
    cropWidth: crop.width * scale,
    cropX: x,
    cropY: y,
    imageWidth: image.width * scale,
    imageX: x + (image.x - crop.x) * scale,
    imageY: y + (image.y - crop.y) * scale,
  };
};
