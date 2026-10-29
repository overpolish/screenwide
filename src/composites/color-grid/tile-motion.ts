// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";

import { motionDurations, motionEasings } from "../../lib/motion";

/**
 * How a row of tiles moves as tiles are kept or forgotten, the way the
 * recording island's and Glide's glyphs do: a tile comes out of a blur from
 * a smaller size and goes back into one (`away`), while the tiles after it
 * slide to their new places (`slide`, spread on every tile's wrapper). The
 * outgoing tile is lifted from the flow (`AnimatePresence`'s `popLayout`)
 * so the rest move at once. Shared by the colour grid and the background
 * tiles, so the two rows of swatches behave alike.
 */
export function useTileMotion() {
  const reducedMotion = useReducedMotion() ?? false;
  const ease = { duration: motionDurations.travel, ease: motionEasings.out };
  return {
    away: reducedMotion
      ? { opacity: 0 }
      : { filter: "blur(2px)", opacity: 0, scale: 0.6 },
    shown: { filter: "blur(0px)", opacity: 1, scale: 1 },
    slide: {
      layout: "position",
      transition: { ...ease, layout: reducedMotion ? { duration: 0 } : ease },
    } as const,
  };
}
