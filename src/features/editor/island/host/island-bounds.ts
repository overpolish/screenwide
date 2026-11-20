// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { type PopupPanelPart } from "../../../popup-panel/api";

/** Where an island's parts reach in the window, `start` its left and `end`
 * its right whatever the text's direction. */
export type Bounds = {
  bottom: number;
  end: number;
  start: number;
  top: number;
};

export const boundsOf = (parts: PopupPanelPart[]): Bounds | null =>
  parts.length === 0
    ? null
    : {
        bottom: Math.max(...parts.map((part) => part.y + part.height)),
        end: Math.max(...parts.map((part) => part.x + part.width)),
        start: Math.min(...parts.map((part) => part.x)),
        top: Math.min(...parts.map((part) => part.y)),
      };

const isSame = (a: Bounds | null, b: Bounds | null) =>
  a === b ||
  (a !== null &&
    b !== null &&
    a.bottom === b.bottom &&
    a.end === b.end &&
    a.start === b.start &&
    a.top === b.top);

/** Where an island stands in the window, kept from the parts it reports, so
 * another island can keep clear of it (`PlacedIsland`'s `clearOf`). Told
 * the same place again, as it is on every frame it is still, it keeps the
 * one it had. */
export function useIslandBounds() {
  const [bounds, setBounds] = useState<Bounds | null>(null);
  return {
    bounds,
    track: (parts: PopupPanelPart[]) => {
      const next = boundsOf(parts);
      setBounds((held) => (isSame(held, next) ? held : next));
    },
  };
}
