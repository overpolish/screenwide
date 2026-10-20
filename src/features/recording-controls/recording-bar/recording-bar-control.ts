// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The recording bar's own icon control: a 48 by 40 landscape box under a
 * 28px glyph, on the panel radius a standalone control of that height takes.
 * Only the bar uses this size, so it lives here rather than as a variant of
 * the icon button. */
export const CAPTURE_CONTROL_CLASS =
  "h-10 w-12 rounded-panel [&_svg.lucide]:size-7";
