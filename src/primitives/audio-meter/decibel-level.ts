// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The quietest level a meter draws; anything below reads as silence. */
export const FLOOR_DECIBELS = -60;

/** How full a meter is for a level in decibels, 0 to 1. The curve is bent so
 * speech sits mid-way: -24 dB fills about half. */
export const decibelLevel = (decibels: number): number => {
  if (decibels < FLOOR_DECIBELS) return 0;
  if (decibels > 0) return 1;
  const normalized = (decibels - FLOOR_DECIBELS) / -FLOOR_DECIBELS;
  return Math.pow(normalized, 1.357);
};
