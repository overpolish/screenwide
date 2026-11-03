// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A unit's box on screen, however it was measured. */
export type UnitRect = {
  bottom: number;
  left: number;
  right: number;
  top: number;
};

/** Two tops within this many logical px count as the same line: words on one
 * line rarely sit flush, and a pause chip's padding moves it a little more. */
const LINE_TOLERANCE = 4;

/**
 * The unit nearest `from` horizontally on the line above (`direction` -1) or
 * below (1), by the boxes `rects` measured it at; `from` itself where there
 * is no such line or no box for it.
 */
export function nearestVerticalUnit(
  rects: readonly (UnitRect | undefined)[],
  from: number,
  direction: 1 | -1,
): number {
  const current = rects[from];
  if (!current) return from;
  const centerX = (current.left + current.right) / 2;
  let line = direction === 1 ? Infinity : -Infinity;
  for (const rect of rects) {
    if (!rect) continue;
    if (
      direction === 1 &&
      rect.top > current.top + LINE_TOLERANCE &&
      rect.top < line
    )
      line = rect.top;
    else if (
      direction === -1 &&
      rect.top < current.top - LINE_TOLERANCE &&
      rect.top > line
    )
      line = rect.top;
  }
  if (!Number.isFinite(line)) return from;
  let best = from;
  let bestDistance = Infinity;
  rects.forEach((rect, index) => {
    if (!rect || Math.abs(rect.top - line) > LINE_TOLERANCE) return;
    const distance = Math.abs((rect.left + rect.right) / 2 - centerX);
    if (distance < bestDistance) {
      bestDistance = distance;
      best = index;
    }
  });
  return best;
}
