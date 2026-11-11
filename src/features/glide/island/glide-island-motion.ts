// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { motionDurations, motionEasings } from "../../../lib/motion";

// The Glide island floats free of the screen's edge, so unlike the recording
// island, which eases into the notch without overshooting, its black springs:
// a little give as it changes shape reads as something alive under the hand.

const fade = { duration: motionDurations.state, ease: motionEasings.out };

/** The black changing shape. */
export const morph = {
  bounce: 0.2,
  type: "spring",
  visualDuration: 0.3,
} as const;

/** The destination following the gesture: quick enough to keep up with a
 * swipe, and settling without a wobble the hand did not make. */
export const follow = {
  bounce: 0.1,
  type: "spring",
  visualDuration: 0.16,
} as const;

/** Coming out where the gesture began: a short fade, holding what it has to
 * show already, so it is there while the hand is still moving. Leaving, it
 * goes out of focus, a little smaller. */
export const emerge = {
  animate: { opacity: 1 },
  exit: {
    filter: "blur(4px)",
    opacity: 0,
    scale: 0.92,
    transition: { duration: motionDurations.travel, ease: motionEasings.out },
  },
  initial: { opacity: 0 },
  transition: fade,
} as const;

/** One look in the black giving way to another as the gesture goes: at
 * once, a short crossfade while the black changes shape round it. Glide
 * follows a hand, so unlike the other islands it never waits for the old
 * look to go before showing the new one. */
export const swap = {
  animate: { opacity: 1 },
  exit: { opacity: 0, transition: fade },
  initial: { opacity: 0 },
  transition: fade,
} as const;

/** A window that will not move: a short shake, the way a refused password
 * answers. */
export const refusal = {
  keyframes: { x: [0, -7, 6, -4, 2, 0] },
  transition: {
    delay: motionDurations.state,
    duration: 0.4,
    ease: motionEasings.out,
  },
};
