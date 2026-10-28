// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Transition, useReducedMotion } from "motion/react";

import { motionDurations, motionEasings } from "../../../lib/motion";

/** How a toggle's glyph moves between its two looks: `move` for a part that
 * travels or turns, with a little spring, as the Animate toggle's circle
 * does; `fade` for a part that comes or goes; `draw` for a line drawn in or
 * out. With reduced motion every change lands at once. */
export function useGlyphMotion(): {
  draw: Transition;
  fade: Transition;
  move: Transition;
} {
  const reduced = useReducedMotion() ?? false;
  if (reduced) {
    const instant = { duration: 0 };
    return { draw: instant, fade: instant, move: instant };
  }
  return {
    draw: { duration: motionDurations.travel, ease: motionEasings.out },
    fade: { duration: motionDurations.state, ease: motionEasings.out },
    move: { bounce: 0.25, type: "spring", visualDuration: 0.3 },
  };
}
