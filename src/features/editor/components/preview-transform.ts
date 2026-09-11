// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Breathing room between the fitted workspace and the viewport edge. */
const VIEWPORT_INSET = 16;

export type PreviewPaneFit = {
  pane: { height: number; width: number; x: number; y: number };
  pixelRatio: number;
  /** On-screen points the pane spends per output pixel at 100% zoom. */
  pointsPerPixel: number;
};

/**
 * Centres `natural` output pixels inside `viewport` points at 100% zoom.
 */
export function fitPreviewPane({
  natural,
  pixelRatio,
  viewport,
}: {
  natural: { height: number; width: number };
  pixelRatio: number;
  viewport: { height: number; width: number };
}): PreviewPaneFit {
  const usableWidth = Math.max(0, viewport.width);
  const pointsPerPixel = Math.min(
    1,
    Math.max(0, usableWidth - VIEWPORT_INSET) / natural.width,
    Math.max(0, viewport.height - VIEWPORT_INSET) / natural.height,
  );
  const width = natural.width * pointsPerPixel;
  const height = natural.height * pointsPerPixel;
  return {
    pane: {
      height,
      width,
      x: (usableWidth - width) / 2,
      y: (viewport.height - height) / 2,
    },
    pixelRatio,
    pointsPerPixel,
  };
}
