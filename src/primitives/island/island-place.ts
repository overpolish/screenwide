// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motionValue, type MotionValue } from "motion/react";
import { useState } from "react";

// Where a host stands an island in its window, carried by `IslandHolder`.

export type IslandPlace = {
  /** Moves the island to `x`, `y` px from where its holder is laid out. */
  move: (x: number, y?: number) => void;
  x: MotionValue<number>;
  y: MotionValue<number>;
};

/** The same place for as long as the host is out. */
export function useIslandPlace(): IslandPlace {
  const [place] = useState(() => {
    const x = motionValue(0);
    const y = motionValue(0);
    return {
      move: (toX: number, toY = 0) => {
        x.set(toX);
        y.set(toY);
      },
      x,
      y,
    };
  });
  return place;
}
