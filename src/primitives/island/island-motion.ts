// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motionDurations, motionEasings } from "../../lib/motion";

// The island moves on the app's motion tokens. What it adds of its own is
// named here: the way its row is revealed, and how long it waits before
// closing after the pointer leaves.

/** How the shape moves, coming out of the notch, going back in and changing
 * between parts: the app's travel motion, quick off the mark and easing into
 * place without overshooting, so a short step feels immediate, a section
 * crossing sides does not swing past its place, and nothing springs out past
 * the notch along the screen's top edge. Every part of the shape shares it,
 * so their edges stay joined mid-way. */
export const glide = {
  duration: motionDurations.travel,
  ease: motionEasings.out,
} as const;

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

/** How a part leaving goes as another takes its place or the parts round
 * it close up: blurred away before the black has finished changing shape
 * round it. */
export const leave = {
  duration: motionDurations.state,
  ease: motionEasings.out,
} as const;

/** How a part arriving in another's place, or where the parts round it make
 * room, is revealed: once the one leaving has gone and what moves has
 * mostly moved, so the two never read as laid over each other. */
export const arrive = {
  ...reveal.transition,
  delay: leave.duration,
} as const;

/** How the island's opacity changes: under a notch it is whole at once, out
 * or tucked away, even when it was laid out without one and the notch came
 * after, as it is then hidden by the notch itself; elsewhere it fades in as
 * it comes out, and out once it has shrunk away. */
export const fade = (isShown: boolean, hasNotch: boolean) => {
  if (hasNotch) return { duration: 0 };
  if (isShown) return { duration: motionDurations.state };
  return { delay: motionDurations.travel, duration: motionDurations.travel };
};

/** How long the island stays open after the pointer leaves, in ms, so a
 * slip, or the island moving as it changes shape, does not slam it shut. */
export const HOVER_LINGER_MS = 350;
