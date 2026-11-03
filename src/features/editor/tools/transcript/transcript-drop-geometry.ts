// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { UnitRect } from "./transcript-line-geometry";

/** Two tops within this many logical px count as the same line, as
 * `nearestVerticalUnit` counts them. */
const LINE_TOLERANCE = 4;

type Point = { x: number; y: number };

/** A boundary between words to drop at: before unit `before`
 * (`rects.length` for after the last), drawn beside unit `beside`, so a
 * boundary where a line wraps shows on the line the pointer is on. */
export type DropBoundary = { before: number; beside: number };

const sameLine = (a: UnitRect, b: UnitRect) =>
  Math.abs(a.top - b.top) <= LINE_TOLERANCE;

/** How far `value` lies outside `[low, high]`; zero inside it. */
const outside = (value: number, low: number, high: number) =>
  Math.max(0, low - value, value - high);

/**
 * The word boundary nearest `point`: the line nearest the pointer, then the
 * unit on it nearest across, and the side of its middle the pointer is on.
 * `isRtl` says whether a unit reads right to left, where its left half is
 * the side after it. Null where no unit has a box.
 */
export function dropBoundary(
  rects: readonly (UnitRect | undefined)[],
  point: Point,
  isRtl: (index: number) => boolean,
): DropBoundary | null {
  let line: UnitRect | null = null;
  let lineDistance = Infinity;
  for (const rect of rects) {
    if (!rect) continue;
    const distance = outside(point.y, rect.top, rect.bottom);
    if (distance < lineDistance) {
      lineDistance = distance;
      line = rect;
    }
  }
  if (!line) return null;
  const lineRect = line;
  let beside = -1;
  let besideDistance = Infinity;
  rects.forEach((rect, index) => {
    if (!rect || !sameLine(rect, lineRect)) return;
    const distance = outside(point.x, rect.left, rect.right);
    if (distance < besideDistance) {
      besideDistance = distance;
      beside = index;
    }
  });
  const rect = rects[beside];
  if (!rect) return null;
  const middle = (rect.left + rect.right) / 2;
  const leading = isRtl(beside) ? point.x > middle : point.x < middle;
  return { before: leading ? beside : beside + 1, beside };
}

/**
 * Where the drop caret stands for `boundary`, in the same coordinates as
 * `rects`: in the middle of the space between two words on one line, and
 * against the word it is drawn beside at the start or end of a line. Null
 * where that word has no box.
 */
export function dropCaret(
  rects: readonly (UnitRect | undefined)[],
  { before, beside }: DropBoundary,
  isRtl: (index: number) => boolean,
): { bottom: number; top: number; x: number } | null {
  const rect = rects[beside];
  if (!rect) return null;
  const leadingOf = (index: number, box: UnitRect) =>
    isRtl(index) ? box.right : box.left;
  const trailingOf = (index: number, box: UnitRect) =>
    isRtl(index) ? box.left : box.right;
  // The word across the boundary from the one it is drawn beside.
  const otherIndex = before === beside ? beside - 1 : beside + 1;
  const other = rects[otherIndex];
  const own =
    before === beside ? leadingOf(beside, rect) : trailingOf(beside, rect);
  const across =
    other && sameLine(other, rect)
      ? before === beside
        ? trailingOf(otherIndex, other)
        : leadingOf(otherIndex, other)
      : own;
  return { bottom: rect.bottom, top: rect.top, x: (own + across) / 2 };
}
