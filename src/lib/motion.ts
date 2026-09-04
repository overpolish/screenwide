// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Shared motion durations in seconds, matching Motion's duration unit. */
export const motionDurations = {
  feedback: 0.06,
  state: 0.12,
  travel: 0.22,
} as const;

export const motionDurationCss = (duration: keyof typeof motionDurations) =>
  `${motionDurations[duration].toString()}s`;

export const motionEasings = {
  out: "easeOut",
} as const;
