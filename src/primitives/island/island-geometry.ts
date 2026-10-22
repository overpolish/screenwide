// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The island's shape in numbers, each set once. The shape is measured and
// animated in script, so these are numbers rather than classes; the two that
// content inside the island also needs reach it as CSS custom properties on
// the island's root (`islandCssVariables`).

/** A screen's notch, in px, as the screen reports it: macOS gives the area
 * either side of it and its height. Leave it out on a screen without one. */
export type IslandNotch = { height: number; width: number };

/** Where under the row a section hangs. */
export type IslandAnchor = "center" | "end" | "start";

export const ISLAND = {
  /** The curved inner corner where the section meets the row. */
  fillet: 14,
  /** The corners once tucked away: the notch's own, or a speck's. */
  hiddenNotchRadius: 10,
  hiddenRadius: 6,
  /** The least row left beside a section; less, and they merge. A step must
   * hold the row's rounded corner and the fillet, or the two curves
   * collide. */
  minStep: 40,
  /** The open shape's corners. Under a notch the row's bottom corners are a
   * little tighter, to sit with the notch's own curve. */
  notchRadius: 20,
  radius: 24,
  /** Between the black's edge and the row's controls. */
  rowInset: 4,
  /** Between the section's edge and what it holds, at the sides and bottom:
   * the row's inset and a list's own, so a list row's corner can follow the
   * section's (`--island-list-radius`). */
  sectionInset: 10,
  /** The hanging section's bottom corners, a little under the row's: the
   * narrower box reads as round as the row with them. */
  sectionRadius: 22,
  /** What the island shrinks to where there is no notch, before it fades. */
  speck: { height: 10, width: 44 },
} as const;

export const px = (value: number) => `${String(value)}px`;

/** The section's curves, in its box: a size container as tall as its black,
 * which reaches up behind the row by `reach`, the row's corner radius. A
 * side that meets the row's end keeps the section's corner whole; it is the
 * row's corner carried down as the section grows, and back up as it folds.
 * On an open side, once what shows below the row is shorter than the corner
 * and a join together, the two shrink in step with it, so the outline stays
 * one smooth S, the join curving in and the corner out, rather than the
 * corner pinching the join to a point. */
export function islandCurves(reach: number) {
  const shown = `(100cqh - ${px(reach)})`;
  const share = (radius: number) =>
    `max(0px, min(${px(radius)}, calc(${shown} * ${String(radius / (ISLAND.fillet + ISLAND.sectionRadius))})))`;
  return {
    fillet: share(ISLAND.fillet),
    meeting: `min(${px(ISLAND.sectionRadius)}, 100cqh)`,
    open: share(ISLAND.sectionRadius),
  };
}

/** What content in the island reads from its root: how far in the section
 * holds it, and the corner a list's last row takes to sit concentric with
 * the section's. */
export const islandCssVariables = {
  "--island-list-radius": px(ISLAND.sectionRadius - ISLAND.sectionInset),
  "--island-section-inset": px(ISLAND.sectionInset),
} as React.CSSProperties;
