// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type GlideIslandShape } from "./glide-island-body";
import { type GlideMapSize } from "./glide-island-map-layout";

// The island's shapes, in px: round with one glyph, and open on the small
// screen.

/** Between the black's edge and the small screen it holds. */
export const MAP_INSET = 6;

/** The app's icon on its destination. */
export const MAP_ICON = 22;

/** A 16:10 screen, its corner concentric with the black's. */
export const MAP: GlideMapSize = {
  gap: 3,
  height: 65,
  radius: 12,
  width: 104,
};

export const MAP_SHAPE: GlideIslandShape = {
  height: MAP.height + MAP_INSET * 2,
  radius: MAP.radius + MAP_INSET,
  width: MAP.width + MAP_INSET * 2,
};

/** Round, holding one glyph or the app's icon. */
export const GLYPH_SHAPE: GlideIslandShape = {
  height: 44,
  radius: 22,
  width: 44,
};
