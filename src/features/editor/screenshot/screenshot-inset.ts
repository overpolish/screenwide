// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  screenshotLayout,
  ScreenshotOutputSettings,
} from "./screenshot-output";

// The padding round a layer's picture, its inset: the frame (the crop rect)
// grown past the picture on every side and filled with the colour found
// behind the content. It is kept as a share of the picture's own shorter
// side, as a scene's inset is, so a new crop or a new size keeps it in
// proportion. The frame is what is drawn; `inset` is what it is grown by
// again whenever the picture changes.

/** The four looks the Screen island offers (None, Tight, Medium, Large), as
 * shares of the picture's shorter side. */
export const INSET_LOOKS = [0, 0.03, 0.06, 0.1] as const;

/** The picture's shorter side in output pixels: the source crop as drawn. */
const shorterSide = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
) => {
  const { sourceCrop } = screenshotLayout(source, settings);
  return Math.max(1, Math.min(sourceCrop.width, sourceCrop.height));
};

/**
 * The layer's padding, as a share of its picture's shorter side.
 *
 * Settings saved while the frame alone carried the padding have no share,
 * and are read from the frame with the picture they are loaded with: half
 * the frame's width past the picture, over the picture's shorter side. A
 * padding that rounds to the pixels a look gave that picture, which is what
 * the island showed it as, is taken as that look, so a padding picked as a
 * look stays one.
 */
export const screenshotInsetShare = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
) => {
  if (settings.inset !== undefined) return settings.inset;
  const { crop, sourceCrop } = screenshotLayout(source, settings);
  const pixels = Math.max(0, (crop.width - sourceCrop.width) / 2);
  const shorter = shorterSide(settings, source);
  return (
    INSET_LOOKS.find(
      (look) => Math.round(look * shorter) === Math.round(pixels),
    ) ?? pixels / shorter
  );
};

/**
 * The layer's frame grown past its picture by `share` of the picture's
 * shorter side on every side: the padded frame the Select tool treats as the
 * layer, with the share kept to grow it by again.
 *
 * Nothing is clamped to the canvas. A padded layer is allowed to run past the
 * frame edges exactly as an unpadded one is, and the pad is filled with the
 * colour detected behind the content rather than cut to fit.
 */
export const insetScreenshotPadding = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
  share: number,
): ScreenshotOutputSettings => {
  const { sourceCrop } = screenshotLayout(source, settings);
  const inset = Math.max(0, share);
  const padding = inset * shorterSide(settings, source);
  return {
    ...settings,
    cropHeight: sourceCrop.height + padding * 2,
    cropWidth: sourceCrop.width + padding * 2,
    cropX: sourceCrop.x - padding,
    cropY: sourceCrop.y - padding,
    inset,
  };
};
