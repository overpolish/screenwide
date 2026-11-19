// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  CaptureSource,
  measureIn,
  percent,
  pixelsPerPoint,
  points,
  snappedInset,
} from "../layer-measure";

import {
  screenshotLayout,
  ScreenshotOutputSettings,
} from "./screenshot-output";

// The padding round a layer's picture, its inset: the frame (the crop rect)
// grown past the picture on every side and filled with the colour found
// behind the content. It is kept in points of the capture, as the layer's
// annotations are, so a look pads every capture alike and a new crop or a
// new size keeps it. The frame is what is drawn; `inset` is what it is
// grown by again whenever the picture changes.

/**
 * The layer's padding in points.
 *
 * Settings saved while the padding was a share of the picture read as the
 * look that share was nearest. Settings saved while the frame alone carried
 * it are read from the frame: half the frame's width past the picture, over
 * the picture's shorter side, as the look that share was nearest.
 */
export const screenshotInsetPoints = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
): number => {
  if (settings.inset)
    return measureIn(snappedInset(settings.inset), "points") ?? 0;
  const { crop, sourceCrop } = screenshotLayout(source, settings);
  const pixels = Math.max(0, (crop.width - sourceCrop.width) / 2);
  const shorter = Math.max(1, Math.min(sourceCrop.width, sourceCrop.height));
  return (
    measureIn(snappedInset(percent((pixels / shorter) * 100)), "points") ?? 0
  );
};

/**
 * The layer's frame grown past its picture by `insetPoints` points of the
 * capture on every side: the padded frame the Select tool treats as the
 * layer, with the padding kept to grow it by again.
 *
 * Nothing is clamped to the canvas. A padded layer is allowed to run past the
 * frame edges exactly as an unpadded one is, and the pad is filled with the
 * colour detected behind the content rather than cut to fit.
 */
export const insetScreenshotPadding = (
  settings: ScreenshotOutputSettings,
  source: CaptureSource,
  insetPoints: number,
): ScreenshotOutputSettings => {
  const { sourceCrop } = screenshotLayout(source, settings);
  const inset = Math.max(0, insetPoints);
  const padding = inset * pixelsPerPoint(settings.imageWidth, source);
  return {
    ...settings,
    cropHeight: sourceCrop.height + padding * 2,
    cropWidth: sourceCrop.width + padding * 2,
    cropX: sourceCrop.x - padding,
    cropY: sourceCrop.y - padding,
    inset: points(inset),
  };
};
