// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext } from "react";

import type { IslandNotch } from "./island-geometry";

/** What the island tells the row it holds: the notch it has to leave room
 * for, if any, and how to ask it to move sideways, in px, so the gap left
 * for the notch sits under it when the row's two sides differ in width. */
export const IslandContext = createContext<{
  notch: IslandNotch | null;
  setShift: (shift: number) => void;
}>({ notch: null, setShift: () => undefined });
