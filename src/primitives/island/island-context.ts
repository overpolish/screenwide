// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use } from "react";

import type { IslandNotch } from "./island-geometry";

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

/** How the section's aside (`IslandAside`) hands the island its element,
 * which the island measures with the section, in the same pass, so the two
 * never disagree for a frame; and which side of the section it opens on:
 * the start for a section hung from the row's end, the end otherwise
 * (`islandLayout`). Nothing outside an island. */
export const IslandAsideContext = createContext<{
  isAtStart: boolean;
  ref: (element: HTMLDivElement | null) => void;
}>({ isAtStart: false, ref: () => undefined });

/** Whether the section's aside opens at its start (`IslandAside`). */
export const useIslandAsideAtStart = () => use(IslandAsideContext).isAtStart;
