// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The Style tab's pop-up widths, kept together so its rows line up: a
 * font's family and weight side by side, and the Template button as wide as
 * the two and the gap between them (36 + 28 spacing units, plus the
 * control gap). Whole class names, so Tailwind finds them.
 */
export const TEXT_FAMILY_WIDTH = "w-36";
export const TEXT_WEIGHT_WIDTH = "w-28";
export const TEXT_FIELD_WIDTH =
  "w-[calc(var(--spacing)*64+var(--spacing-control))]";
