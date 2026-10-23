// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext } from "react";

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
