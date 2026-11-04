// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The Frame island's narrowest width. The background tiles wrap to the
 * island rather than set its width, and the colours' line under them, which
 * comes for a mesh or a colour of your own, is widest for a mesh: Mesh by
 * name, its four colours and Randomise. The island never narrows past that
 * line, so choosing a mesh or a solid colour leaves it, and the tiles'
 * rows, as they were. */
export const FRAME_ISLAND_WIDTH = {
  /** 232 px. */
  section: "min-w-58",
} as const;
