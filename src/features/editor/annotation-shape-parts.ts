// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The pieces a stored annotation's shape is read from, each held to what the
 * compositor can draw: a point, a text box's pointer, a highlight's band and
 * the page it was read from.
 */

import type {
  AnnotationHighlight,
  AnnotationPoint,
  HighlightBand,
  TextPointer,
} from "./annotations";

/** A stored point, or null where it is not a finite one. */
export const annotationPoint = (value: unknown): AnnotationPoint | null => {
  const point = value as Partial<AnnotationPoint> | null | undefined;
  return typeof point?.x === "number" &&
    typeof point.y === "number" &&
    Number.isFinite(point.x) &&
    Number.isFinite(point.y)
    ? { x: point.x, y: point.y }
    : null;
};

/** A stored band, or null where it cannot be drawn. */
export const highlightBand = (value: unknown): HighlightBand | null => {
  const band = (value ?? {}) as Partial<HighlightBand>;
  const sides = [band.left, band.top, band.right, band.bottom];
  if (!sides.every((side) => typeof side === "number" && Number.isFinite(side)))
    return null;
  const [left, top, right, bottom] = sides as number[];
  return {
    bottom: Math.max(top, bottom),
    left: Math.min(left, right),
    right: Math.max(left, right),
    top: Math.min(top, bottom),
  };
};

/** How bright a stored page and its ink are, or dark ink on a light page -
 * the twin of `HighlightTone::default` - where that cannot be read. */
export const highlightTone = (value: unknown): AnnotationHighlight["tone"] => {
  const tone = (value ?? {}) as Partial<AnnotationHighlight["tone"]>;
  const share = (level: unknown, fallback: number) =>
    typeof level === "number" && Number.isFinite(level)
      ? Math.min(1, Math.max(0, level))
      : fallback;
  return { ink: share(tone.ink, 0), surface: share(tone.surface, 1) };
};

/** Where a fresh text box's pointer sits: tucked into the middle of its
 * bottom edge. The twin of `TextPointer::default`. */
const TUCKED_POINTER: TextPointer = {
  along: { x: 0, y: 1 },
  reach: { x: 0, y: 0 },
};

/** A stored pointer, or the tucked one where what is stored cannot be read. */
export const textPointer = (value: unknown): TextPointer => {
  const pointer = (value ?? {}) as Partial<TextPointer>;
  const along = annotationPoint(pointer.along);
  const reach = annotationPoint(pointer.reach);
  return along && reach
    ? {
        along: {
          x: Math.min(1, Math.max(-1, along.x)),
          y: Math.min(1, Math.max(-1, along.y)),
        },
        reach: { x: Math.max(0, reach.x), y: Math.max(0, reach.y) },
      }
    : TUCKED_POINTER;
};
