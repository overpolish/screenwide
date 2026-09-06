// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The zoom ceiling every ordinary preview keeps, mirroring the fixed floor the
 * native workspace editor clamps its own pan/zoom against.
 */
export const MINIMUM_ZOOM_CEILING = 16;

/** How far past 100% actual pixels the content-aware ceiling reaches. */
const NATIVE_PIXEL_ZOOM_HEADROOM = 4;

/** Breathing room between the fitted workspace and the viewport edge. */
const VIEWPORT_INSET = 16;

export type PreviewPaneFit = {
  pane: { height: number; width: number; x: number; y: number };
  pixelRatio: number;
  /** On-screen points the pane spends per output pixel at 100% zoom. */
  pointsPerPixel: number;
};

/** Centres `natural` output pixels inside `viewport` points at 100% zoom. */
export function fitPreviewPane({
  natural,
  pixelRatio,
  viewport,
}: {
  natural: { height: number; width: number };
  pixelRatio: number;
  viewport: { height: number; width: number };
}): PreviewPaneFit {
  const pointsPerPixel = Math.min(
    1,
    Math.max(0, viewport.width - VIEWPORT_INSET) / natural.width,
    Math.max(0, viewport.height - VIEWPORT_INSET) / natural.height,
  );
  const width = natural.width * pointsPerPixel;
  const height = natural.height * pointsPerPixel;
  return {
    pane: {
      height,
      width,
      x: (viewport.width - width) / 2,
      y: (viewport.height - height) / 2,
    },
    pixelRatio,
    pointsPerPixel,
  };
}

/**
 * The zoom ceiling for a pane fitted as `fit` describes.
 *
 * 100% is fit-to-pane, so one output pixel covers one physical screen pixel at
 * `1 / (pointsPerPixel * pixelRatio)`. A scrolling capture tens of thousands of
 * pixels tall fits so small that the fixed ceiling lands far below that, and
 * the user could never inspect the capture at actual pixels; the ceiling
 * therefore follows the content, and never drops under the fixed one.
 */
export function maximumZoom({ pixelRatio, pointsPerPixel }: PreviewPaneFit) {
  if (!(pointsPerPixel > 0) || !(pixelRatio > 0)) return MINIMUM_ZOOM_CEILING;
  return Math.max(
    MINIMUM_ZOOM_CEILING,
    NATIVE_PIXEL_ZOOM_HEADROOM / (pointsPerPixel * pixelRatio),
  );
}
