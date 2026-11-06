// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type GlideRect, regionRect, shortfall } from "../glide-geometry";

import type { GlideIslandState } from "./glide-island-types";

/** A small screen's size in px, its corner radius included. */
export type GlideMapSize = {
  height: number;
  radius: number;
  width: number;
};

/** A rectangle on the small screen, in px from its top start corner, with
 * the radius of each of its corners. */
export type GlideMapBox = {
  height: number;
  radius: {
    bottomEnd: number;
    bottomStart: number;
    topEnd: number;
    topStart: number;
  };
  width: number;
  x: number;
  y: number;
};

/** The Dock a minimize sends the window to: a bar along the screen's foot,
 * lifted clear of it. */
const DOCK = { height: 5, lift: 3, width: 1 / 3 };

/** A window's own corner on the small screen, in px: a real window's, scaled
 * down, reads as next to none. */
const WINDOW_RADIUS = 3;

/** How near a screen's edge a box counts as on it, in px. */
const EDGE = 0.5;

type Box = Omit<GlideMapBox, "radius">;

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
  const { height, radius, width } = size;
  // Round as the screen's corner where a box stands in one, so the two are
  // concentric; a window's own small corner wherever else.
  const corner = (isAtStart: boolean, isAtTop: boolean, box: Box) => {
    const atStart = isAtStart
      ? box.x <= EDGE
      : box.x + box.width >= width - EDGE;
    const atTop = isAtTop ? box.y <= EDGE : box.y + box.height >= height - EDGE;
    return atStart && atTop ? radius : WINDOW_RADIUS;
  };
  // Edge to edge, as the window will stand: a gap round the destination
  // made it read smaller than the share of the screen it takes.
  const place = (rect: GlideRect): GlideMapBox => {
    const box = {
      height: rect.height * height,
      width: rect.width * width,
      x: rect.x * width,
      y: rect.y * height,
    };
    return {
      ...box,
      radius: {
        bottomEnd: corner(false, false, box),
        bottomStart: corner(true, false, box),
        topEnd: corner(false, true, box),
        topStart: corner(true, true, box),
      },
    };
  };

  if (pending === "minimize") {
    const dockWidth = width * DOCK.width;
    const round = DOCK.height / 2;
    const tile = {
      height: DOCK.height,
      radius: {
        bottomEnd: round,
        bottomStart: round,
        topEnd: round,
        topStart: round,
      },
      width: dockWidth,
      x: (width - dockWidth) / 2,
      y: height - DOCK.lift - DOCK.height,
    };
    return {
      focus: centre(tile),
      isMinimizing: true,
      reached: null,
      requested: null,
      tile,
    };
  }
  if (!region) return null;

  const short = shortfall(fit, region, pending);
  if (!short) {
    const tile = place(regionRect(region));
    return {
      focus: centre(tile),
      isMinimizing: false,
      reached: null,
      requested: null,
      tile,
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
  };
}
