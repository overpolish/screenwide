// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The corners a whole picture is offered, the canvas or the screenshot in
 * it: square, a soft rounding, and a rounder one. A shape's quarter or half
 * would round a picture into a capsule, so the radii are smaller; the
 * glyphs are the ones every corner choice draws, square, rounded and round,
 * so the three looks read the same on every island. */
export const PICTURE_CORNERS = [
  { glyph: 0, radius: 0 },
  { glyph: 25, radius: 4 },
  { glyph: 50, radius: 12 },
] as const;
