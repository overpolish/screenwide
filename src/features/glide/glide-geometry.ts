// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type GlideAction } from "./detection/glide-detection";
import { glideGridRows, type GlideRegion } from "./detection/glide-regions";

/** A rectangle in work-area fractions. */
export type GlideRect = { height: number; width: number; x: number; y: number };

/** What the last settle achieved, in work-area fractions. */
export type GlideFit = { actual: GlideRect; fits: boolean };

export const regionRect = (region: GlideRegion): GlideRect => ({
  height: region.rowSpan / glideGridRows,
  width: region.colSpan / region.gridCols,
  x: region.colStart / region.gridCols,
  y: region.rowStart / glideGridRows,
});

/** The part of the actual frame that satisfied the requested destination. */
const intersect = (first: GlideRect, second: GlideRect): GlideRect | null => {
  const x = Math.max(first.x, second.x);
  const y = Math.max(first.y, second.y);
  const right = Math.min(first.x + first.width, second.x + second.width);
  const bottom = Math.min(first.y + first.height, second.y + second.height);
  if (right <= x || bottom <= y) return null;
  return { height: bottom - y, width: right - x, x, y };
};

/**
 * An app that could not fill its region: the frame it reached, and the part
 * of that frame inside the region. Null while it fits, before anything has
 * settled, and while an armed action shows its own hint.
 */
export const shortfall = (
  fit: GlideFit | null,
  region: GlideRegion | null,
  pending: GlideAction | null,
) => {
  if (!fit || fit.fits || !region || pending !== null) return null;
  return {
    actual: fit.actual,
    overlap: intersect(regionRect(region), fit.actual),
  };
};
