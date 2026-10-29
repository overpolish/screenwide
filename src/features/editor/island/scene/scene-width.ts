// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The scene island's width. Its groups swap under one row, and the
 * layout tiles scroll rather than set a width of their own, so the island
 * holds one width whichever group is open, which fits within its narrowest
 * row and keeps the row's title and buttons still: a part hanging in the
 * section, which the section's inset of 10 px frames at each side to a
 * black 260 px wide. */
export const SCENE_ISLAND_WIDTH = {
  /** 240 px. */
  part: "w-60",
} as const;
