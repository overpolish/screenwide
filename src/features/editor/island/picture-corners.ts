// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CORNER_LOOKS } from "../layer-measure";

/** The corners the canvas is offered, in percent of its shorter side:
 * square, a soft rounding, and a rounder one. A shape's quarter or half
 * would round a picture into a capsule, so the radii are smaller; the
 * glyphs are the ones every corner choice draws, square, rounded and round,
 * so the three looks read the same on every island. */
export const PICTURE_CORNERS = [
  { glyph: 0, radius: 0 },
  { glyph: 25, radius: 4 },
  { glyph: 50, radius: 12 },
] as const;

/** The same three looks for a screen or screenshot, in points of the
 * capture, so a look rounds every capture alike and matches the corners of
 * the windows in it. */
export const CAPTURE_CORNERS = [
  { glyph: 0, radius: CORNER_LOOKS[0] },
  { glyph: 25, radius: CORNER_LOOKS[1] },
  { glyph: 50, radius: CORNER_LOOKS[2] },
] as const;
