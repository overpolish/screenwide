// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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

/** Coming out where the gesture began, out of a blur from a smaller size, and
 * leaving the same way, a shorter distance. */
export const emerge = {
  animate: { filter: "blur(0px)", opacity: 1, scale: 1 },
  exit: {
    filter: "blur(4px)",
    opacity: 0,
    scale: 0.92,
    transition: { duration: motionDurations.travel, ease: motionEasings.out },
  },
  initial: { filter: "blur(6px)", opacity: 0, scale: 0.6 },
  transition: { ...morph, filter: fade, opacity: fade },
} as const;

/** One look in the black giving way to another: the new one clears into
 * focus a beat after the shape starts to change, so the shape leads. */
export const swap = {
  animate: { filter: "blur(0px)", opacity: 1, scale: 1 },
  exit: {
    filter: "blur(4px)",
    opacity: 0,
    scale: 0.96,
    transition: fade,
  },
  initial: { filter: "blur(6px)", opacity: 0, scale: 0.92 },
  transition: {
    delay: motionDurations.feedback,
    duration: motionDurations.travel,
    ease: motionEasings.out,
  },
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

/** The hand has rested and the next gesture will land: the black takes a
 * breath, the visual sibling of the haptic tick. */
export const breath = {
  keyframes: { scale: [1, 1.04, 1] },
  transition: { duration: motionDurations.travel, ease: motionEasings.out },
};
