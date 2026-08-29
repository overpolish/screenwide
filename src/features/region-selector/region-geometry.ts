// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Region } from "../recording-sources/types";

const wholePixel = (value: number) => Math.round(value);
export const wholePixelSize = (value: number) => Math.max(1, wholePixel(value));

/** The "no region yet" region a screenshot session starts from. */
export const EMPTY_REGION: Region = {
  position: { x: 0, y: 0 },
  size: { height: 0, width: 0 },
};

/** The region on whole-pixel boundaries, as capture and storage want it. */
export const snapRegion = (region: Region): Region => ({
  position: {
    x: wholePixel(region.position.x),
    y: wholePixel(region.position.y),
  },
  size: {
    height: wholePixelSize(region.size.height),
    width: wholePixelSize(region.size.width),
  },
});

export const fitRegion = (
  region: Region,
  width: number,
  height: number,
): Region => {
  const margin = 20;
  const fittedWidth = wholePixelSize(
    Math.min(region.size.width, width - margin),
  );
  const fittedHeight = wholePixelSize(
    Math.min(region.size.height, height - margin),
  );
  return {
    position: {
      x: wholePixel(
        Math.max(0, Math.min(region.position.x, width - fittedWidth)),
      ),
      y: wholePixel(
        Math.max(0, Math.min(region.position.y, height - fittedHeight)),
      ),
    },
    size: { height: fittedHeight, width: fittedWidth },
  };
};
