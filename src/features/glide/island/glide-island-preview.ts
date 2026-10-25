// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cell, IDLE } from "./glide-island-script";

import type { GlideIslandState } from "./glide-island-types";
import type { GlideRegion } from "../detection/glide-regions";
import type { GlideFit } from "../glide-geometry";

// The states the island's stories show, and every region it draws.

export const gliding = (
  region: GlideRegion | null,
  fit: GlideFit | null = null,
): GlideIslandState => ({ ...IDLE, fit, phase: "gliding", region });

const halves = cell({ gridCols: 2 });
const thirds = cell({ gridCols: 3 });
const quarter = cell({ gridCols: 2, rowSpan: 1 });
const rightHalf = { ...halves, colStart: 1 };

/** Every region the gesture reaches, grouped as the grids draw them. */
export const REGIONS = {
  fullScreen: [{ ...halves, colSpan: 2 }],
  halves: [halves, rightHalf],
  quarters: [
    quarter,
    { ...quarter, colStart: 1 },
    { ...quarter, rowStart: 1 },
    { ...quarter, colStart: 1, rowStart: 1 },
  ],
  rows: [
    { ...quarter, colSpan: 2 },
    { ...quarter, colSpan: 2, rowStart: 1 },
  ],
  thirds: [thirds, { ...thirds, colStart: 1 }, { ...thirds, colStart: 2 }],
  twoThirds: [
    { ...thirds, colSpan: 2 },
    { ...thirds, colSpan: 2, colStart: 1 },
  ],
} satisfies Record<string, GlideRegion[]>;

/** An app that stops short of the right half's width. */
export const NARROWER = gliding(rightHalf, {
  actual: { height: 1, width: 0.3, x: 0.7, y: 0 },
  fits: false,
});

/** A window wider than the right half, spilling over its edge. */
export const WIDER = gliding(rightHalf, {
  actual: { height: 1, width: 0.7, x: 0.3, y: 0 },
  fits: false,
});

export const MINIMIZE: GlideIslandState = {
  ...gliding(halves),
  pending: "minimize",
};

/** The island before the gesture has a destination. */
export const WAITING: GlideIslandState = { ...IDLE, phase: "gliding" };

/** The island as the window is sent. */
export const SENT: GlideIslandState = { ...IDLE, phase: "committed" };
