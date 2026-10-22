// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motionDurations, motionEasings } from "../../lib/motion";

// The island moves on the app's motion tokens. What it adds of its own is
// named here: the spring it comes out of the notch on, the way its row is
// revealed, and how long it waits before closing after the pointer leaves.

/** Coming out of the notch and going back in, the shape springs a little
 * past its size and settles, as the island does. */
export const morph = { bounce: 0.25, duration: 0.5, type: "spring" } as const;

/** Out, it changes shape between parts with the app's travel motion: quick
 * off the mark and easing into place without overshooting, so a short step
 * feels immediate and a section crossing sides does not swing past its
 * place. Every part of the shape shares one curve, so their edges stay
 * joined mid-way. */
export const glide = {
  duration: motionDurations.travel,
  ease: motionEasings.out,
} as const;

export type Curve = typeof morph | typeof glide;

/** What the island holds coming into focus as it is shown, and leaving it
 * as it goes: lifted out of a blur rather than cut in. */
export const focusIn = {
  animate: { filter: "blur(0px)", opacity: 1, scale: 1 },
  exit: { filter: "blur(4px)", opacity: 0, scale: 0.96 },
  transition: { duration: motionDurations.travel, ease: motionEasings.out },
} as const;

/** A half of the row revealed where it sits, as though the glass were
 * clearing: deeper than `focusIn`, and a beat after the black has begun to
 * open, so the shape leads and its content follows. */
export const reveal = {
  animate: { filter: "blur(0px)", opacity: 1, scale: 1 },
  initial: { filter: "blur(8px)", opacity: 0, scale: 0.94 },
  transition: {
    delay: motionDurations.feedback,
    duration: motionDurations.travel + motionDurations.feedback * 2,
    ease: motionEasings.out,
  },
} as const;

/** How long the island stays open after the pointer leaves, in ms, so a
 * slip, or the island moving as it changes shape, does not slam it shut. */
export const HOVER_LINGER_MS = 350;
