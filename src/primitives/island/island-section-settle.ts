// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type MotionValue, useMotionValue, useTransform } from "motion/react";
import { useLayoutEffect } from "react";

import { px } from "./island-geometry";

// Around a notch, what the section holds goes straight to where it will rest
// while the black glides there round it, as the row's halves do. When the
// row grows on one side, say for controls under the pointer, the island
// moves sideways to keep the notch over its gap and the section's box moves
// back by as much from the island's middle: the side that did not change
// stays put. What hangs is held against the island's sideways move at once
// (`sectionX`); this holds it against the box's move too, by placing it from
// where the box is going rather than where it is, so the two cancel and
// nothing travels. Only once the section is open in its place: opening and
// folding keep their own choreography. Without a notch the island grows
// evenly to both sides and what hangs eases along with it.

export function useSectionSettle({
  boxStart,
  isActive,
  target,
}: {
  /** The box's place from the island's middle, as it is animated. */
  boxStart: MotionValue<string>;
  /** Around a notch, with the section open in its place. */
  isActive: boolean;
  /** Where the box is going. */
  target: number;
}) {
  /** Where the box is going, while the section settles there; none
   * otherwise. */
  const settleTo = useMotionValue(Number.NaN);
  useLayoutEffect(() => {
    settleTo.set(isActive ? target : Number.NaN);
  }, [isActive, settleTo, target]);
  return useTransform(() => {
    const to = settleTo.get();
    return px(Number.isNaN(to) ? 0 : to - parseFloat(boxStart.get()));
  });
}
