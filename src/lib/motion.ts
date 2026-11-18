// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Shared motion durations in seconds, matching Motion's duration unit. */
export const motionDurations = {
  feedback: 0.06,
  /** Many things moving at once to new places, slow enough to follow
   * which went where, such as a timeline making room for what is moved. */
  rearrange: 0.4,
  /** The picture taking a setting chosen on an island, such as its padding
   * or corners, so the change is seen rather than jumped to. */
  reshape: 0.28,
  state: 0.12,
  travel: 0.22,
} as const;

export const motionDurationCss = (duration: keyof typeof motionDurations) =>
  `${motionDurations[duration].toString()}s`;

export const motionEasings = {
  out: "easeOut",
} as const;

/** Arriving out of a blur rather than cutting in, as a Settings pane or a
 * revealed part does. The filter is dropped once clear, so it leaves no
 * layer behind. */
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
