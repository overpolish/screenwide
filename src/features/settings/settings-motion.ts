// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motionDurations, motionEasings } from "../../lib/motion";

// How Settings moves, on the app's motion tokens and the islands' way of
// showing things: what arrives clears out of a blur rather than cutting in.
// The window sets reduced motion to follow the system (`MotionConfig`),
// which keeps the fades and drops the movement.

/** Arriving out of a blur: a pane opened, or a row's picture. The filter is
 * dropped once clear, so it leaves no layer behind. */
export const clearIn = {
  animate: {
    filter: "blur(0px)",
    opacity: 1,
    transitionEnd: { filter: "none" },
  },
  exit: { filter: "blur(6px)", opacity: 0 },
  initial: { filter: "blur(6px)", opacity: 0 },
  transition: { duration: motionDurations.travel, ease: motionEasings.out },
} as const;
