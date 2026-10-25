// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type GlideRect, regionRect, shortfall } from "../glide-geometry";

import type { GlideIslandState } from "./glide-island-types";

/** A small screen's size in px: the gap keeps a destination clear of the
 * screen's edge and its corner concentric with the screen's. */
export type GlideMapSize = {
  gap: number;
  height: number;
  radius: number;
  width: number;
};

/** A rectangle on the small screen, in px from its top start corner. */
export type GlideMapBox = {
  height: number;
  width: number;
  x: number;
  y: number;
};

/** The Dock a minimize sends the window to: a bar along the screen's foot. */
const DOCK = { height: 5, width: 1 / 3 };

const centre = (box: GlideMapBox) => ({
  x: box.x + box.width / 2,
  y: box.y + box.height / 2,
});

/**
 * Where everything on the small screen goes: the destination in primary, the
 * region and frame of an app that could not fill it in neutral, and the point
 * the app's icon names. Null with no destination to draw.
 */
export function layOutMap(
  size: GlideMapSize,
  {
    fit,
    pending,
    region,
  }: Pick<GlideIslandState, "fit" | "pending" | "region">,
) {
  const { gap, height, width } = size;
  const place = (rect: GlideRect): GlideMapBox => ({
    height: rect.height * height - gap * 2,
    width: rect.width * width - gap * 2,
    x: rect.x * width + gap,
    y: rect.y * height + gap,
  });

  if (pending === "minimize") {
    const dockWidth = width * DOCK.width;
    const tile = {
      height: DOCK.height,
      width: dockWidth,
      x: (width - dockWidth) / 2,
      y: height - gap - DOCK.height,
    };
    return {
      focus: centre(tile),
      isMinimizing: true,
      reached: null,
      requested: null,
      tile,
      tileRadius: DOCK.height / 2,
    };
  }
  if (!region) return null;

  const tileRadius = size.radius - gap;
  const short = shortfall(fit, region, pending);
  if (!short) {
    const tile = place(regionRect(region));
    return {
      focus: centre(tile),
      isMinimizing: false,
      reached: null,
      requested: null,
      tile,
      tileRadius,
    };
  }
  // Only the part of the frame inside the region is primary, so a window
  // that came up short and one that overflowed read the same way.
  const reached = place(short.actual);
  const tile = short.overlap ? place(short.overlap) : null;
  return {
    focus: centre(tile ?? reached),
    isMinimizing: false,
    reached,
    requested: place(regionRect(region)),
    tile,
    tileRadius,
  };
}
