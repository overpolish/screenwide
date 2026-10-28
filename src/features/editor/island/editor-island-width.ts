// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The widths an editor island's parts are laid out at, so the black keeps
 * one width whichever part is in hand: a part hanging in the section, which
 * the section's inset of 10 px frames at each side, or a line under the row,
 * which its own inset of 4 px frames. Both come to a black 260 px wide. */
export const EDITOR_ISLAND_WIDTH = {
  /** 252 px. */
  line: "w-63",
  /** 240 px. */
  part: "w-60",
} as const;
