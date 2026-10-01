// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type {
  Annotation,
  AnnotationHighlight,
  AnnotationPoint,
  AnnotationStyle,
  HighlightBand,
} from "./annotations";

/** How far apart strokes are laid, as a share of their width. The twin of
 * `STRIDE` in `src-tauri/src/editor/annotations/highlight/manual.rs`. */
const STRIDE = 0.92;

/**
 * The strokes a highlight laid by hand covers the box between `start` and
 * `end` with, top to bottom, each `width` tall and spanning the box. A drag
 * no taller than one stroke is one stroke, centred on the drag. The twin
 * of `strokes` in `manual.rs`, which lays them while a gesture draws; this
 * lays them again when the panel changes a drawn one.
 */
export const highlightStrokes = (
  start: AnnotationPoint,
  end: AnnotationPoint,
  width: number,
): HighlightBand[] => {
  const marker = Number.isFinite(width) ? Math.max(1, width) : 1;
  const left = Math.min(start.x, end.x);
  const right = Math.max(start.x, end.x);
  const top = Math.min(start.y, end.y);
  const bottom = Math.max(start.y, end.y);
  const stroke = (at: number) => ({
    bottom: at + marker,
    left,
    right,
    top: at,
  });
  const tall = bottom - top;
  if (tall <= marker) return [stroke((top + bottom - marker) / 2)];
  const count = Math.ceil((tall - marker) / (marker * STRIDE)) + 1;
  const step = (tall - marker) / (count - 1);
  return Array.from({ length: count }, (_, index) =>
    stroke(top + step * index),
  );
};

/**
 * A highlight's shape once the panel dresses it in `style`, where that lays
 * it again: switched to being laid by hand, it becomes a box of strokes over
 * the space between its ends, with a marker as tall as its first line. It
 * keeps the tone its text was read with until an end is moved, which reads
 * the page under the box. Null where the shape stays as it is. Switched back
 * to fitting text, the box stays until an end is moved, which selects again
 * from the picture.
 */
export const relaidHighlight = (
  annotation: Annotation,
  style: AnnotationStyle,
): AnnotationHighlight | null => {
  const shape = annotation.shape;
  if (shape.kind !== "highlight" || !style.manual || annotation.style.manual)
    return null;
  const first = shape.bands[0] as HighlightBand | undefined;
  const tall = first ? first.bottom - first.top : 0;
  if (!(tall > 0)) return null;
  return {
    ...shape,
    bands: highlightStrokes(shape.start, shape.end, tall),
  };
};
