// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  canvasShare,
  canvasShortest,
  canvasValue,
  CaptureSource,
  percent,
  snappedInset,
} from "../layer-measure";

import {
  screenshotLayout,
  ScreenshotOutputSettings,
} from "./screenshot-output";

// The padding round a layer's picture, its inset: the frame (the crop rect)
// grown past the picture on every side and filled with the colour found
// behind the content. It is a share of the canvas's shorter side, so a look
// pads every capture alike and a new crop or a new size keeps it. The frame
// is what is drawn; `inset` is what it is grown by again whenever the
// picture changes.

/**
 * The layer's padding in percent of the canvas's shorter side.
 *
 * Settings saved while the padding was a share of the picture read as the
 * look that share was nearest. Settings saved while the frame alone carried
 * it are read from the frame: half the frame's width past the picture, over
 * the picture's shorter side, as the look that share was nearest.
 */
export const screenshotInset = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
): number => {
  if (settings.inset) return canvasValue(snappedInset(settings.inset)) ?? 0;
  const { crop, sourceCrop } = screenshotLayout(source, settings);
  const pixels = Math.max(0, (crop.width - sourceCrop.width) / 2);
  const shorter = Math.max(1, Math.min(sourceCrop.width, sourceCrop.height));
  return canvasValue(snappedInset(percent((pixels / shorter) * 100))) ?? 0;
};

/**
 * The layer's frame grown past its picture by `inset` percent of the
 * canvas's shorter side on every side: the padded frame the Select tool
 * treats as the layer, with the padding kept to grow it by again.
 *
 * Nothing is clamped to the canvas. A padded layer is allowed to run past the
 * frame edges exactly as an unpadded one is, and the pad is filled with the
 * colour detected behind the content rather than cut to fit.
 */
export const insetScreenshotPadding = (
  settings: ScreenshotOutputSettings,
  source: CaptureSource,
  inset: number,
): ScreenshotOutputSettings => {
  const { sourceCrop } = screenshotLayout(source, settings);
  const share = Math.max(0, inset);
  const padding = (share * canvasShortest(settings)) / 100;
  return {
    ...settings,
    cropHeight: sourceCrop.height + padding * 2,
    cropWidth: sourceCrop.width + padding * 2,
    cropX: sourceCrop.x - padding,
    cropY: sourceCrop.y - padding,
    inset: canvasShare(share),
  };
};
