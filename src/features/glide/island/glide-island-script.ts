// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type GlideRegion } from "../detection/glide-regions";
import { type Beat, usePlayback } from "../glide-playback";

import type { GlideIslandState } from "./glide-island-types";

// The island's gestures: a session starting, destinations as the hand moves,
// the move's own answer once it settles, and the end.

export const IDLE: GlideIslandState = {
  fit: null,
  pending: null,
  phase: "idle",
  region: null,
};

/** A full-height cell on the given grid, refined per beat. */
export const cell = (
  cells: Partial<GlideRegion> & Pick<GlideRegion, "gridCols">,
): GlideRegion => ({
  colSpan: 1,
  colStart: 0,
  rowSpan: 2,
  rowStart: 0,
  ...cells,
});

const rightHalf = cell({ colStart: 1, gridCols: 2 });

const scripts = {
  /** Halves, along the thirds, into a corner, an app that cannot fill the
   * right half, an armed minimize turned into the bottom row, then a fill. */
  gesture: [
    { after: 700, icon: false, state: { ...IDLE, phase: "gliding" } },
    { after: 180, icon: true },
    { after: 320, state: { region: cell({ gridCols: 2 }) } },
    { after: 420, rest: true },
    { after: 700, state: { region: cell({ colSpan: 2, gridCols: 3 }) } },
    { after: 450, state: { region: cell({ colStart: 1, gridCols: 3 }) } },
    {
      after: 450,
      state: { region: cell({ colSpan: 2, colStart: 1, gridCols: 3 }) },
    },
    { after: 450, state: { region: cell({ colStart: 2, gridCols: 3 }) } },
    {
      after: 600,
      state: { region: cell({ colStart: 2, gridCols: 3, rowSpan: 1 }) },
    },
    { after: 600, state: { region: rightHalf } },
    {
      after: 350,
      state: {
        fit: { actual: { height: 1, width: 0.3, x: 0.7, y: 0 }, fits: false },
      },
    },
    { after: 1100, state: { pending: "minimize" } },
    {
      after: 1000,
      state: {
        fit: null,
        pending: null,
        region: cell({ colSpan: 2, gridCols: 2, rowSpan: 1, rowStart: 1 }),
      },
    },
    { after: 600, state: { region: cell({ colSpan: 2, gridCols: 2 }) } },
    { after: 380, rest: true },
    { after: 800, state: { phase: "committed" } },
    { after: 700, state: IDLE },
  ],
  /** A window the platform will not let us place: the lock, held a beat. */
  locked: [
    { after: 700, icon: false, state: { ...IDLE, phase: "locked" } },
    { after: 1400, state: IDLE },
  ],
} satisfies Record<string, Beat<GlideIslandState>[]>;

export type GlideIslandScript = keyof typeof scripts;

export const useScript = (name: GlideIslandScript | undefined) =>
  usePlayback<GlideIslandState>(name ? scripts[name] : undefined, IDLE);
