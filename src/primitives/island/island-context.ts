// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext } from "react";

import type { IslandNotch } from "./island-geometry";
import type { MotionValue } from "motion/react";

/** What the island tells the row it holds: the notch it has to leave room
 * for, if any; how to ask it to move sideways, in px, so the gap left for
 * the notch sits under it when the row's two sides differ in width; and
 * whether it is out, so the row can fold into the notch as it tucks away. */
export const IslandContext = createContext<{
  isShown: boolean;
  notch: IslandNotch | null;
  setShift: (shift: number) => void;
}>({
  isShown: true,
  notch: null,
  setShift: () => undefined,
});

/** What the section tells the parts it swaps between: a part coming in calls
 * `noteSwap` as it mounts, and a move of the section's box that follows the
 * swap holds the parts swapping still, by `offset`, rather than carrying
 * them along; whatever else the section holds rides with the box. None
 * outside a section. */
export const IslandSectionContext = createContext<{
  noteSwap: () => void;
  offset: MotionValue<string> | null;
}>({
  noteSwap: () => undefined,
  offset: null,
});
