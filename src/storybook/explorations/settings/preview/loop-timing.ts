// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Timing for the looping pictures, each read from one clock (`useClock`).

export const lerp = (from: number, to: number, amount: number) =>
  from + (to - from) * amount;

const ease = (amount: number) => amount * amount * (3 - 2 * amount);

/** How far into the part of a loop from `start` to `end` a time is, 0 to 1,
 * eased in and out. */
export const progress = (at: number, start: number, end: number) =>
  ease(Math.min(Math.max((at - start) / (end - start), 0), 1));
