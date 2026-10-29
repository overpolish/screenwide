// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The moment island's widths. A waveform and what was said have no width
 * of their own, so the island sets one: the player's line, which its inset
 * of 4 px frames, and the transcript under it in the section, which the
 * section's inset of 10 px frames. Both come to a black 260 px wide. */
export const MOMENT_ISLAND_WIDTH = {
  /** 252 px. */
  line: "w-63",
  /** 240 px. */
  part: "w-60",
} as const;
