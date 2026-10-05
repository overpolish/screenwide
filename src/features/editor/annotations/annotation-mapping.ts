// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotationPoint, AnnotationShape } from "./annotations";

/**
 * The same shape with every point moved by `map`, the twin of
 * `AnnotationShape::mapped` in `src-tauri/src/editor/annotations/shape.rs`.
 * What a shape *is* rides through untouched - a counter's angle and number,
 * a text box's words and its pointer, which is held against the box, a
 * image's turn and picture - because every space an annotation travels
 * between keeps the picture's aspect. A length is carried as far as `map`
 * stretches it.
 */
export const mappedAnnotationShape = (
  shape: AnnotationShape,
  map: (point: AnnotationPoint) => AnnotationPoint,
): AnnotationShape => {
  switch (shape.kind) {
    case "arrow":
      return {
        ...shape,
        control: map(shape.control),
        end: map(shape.end),
        start: map(shape.start),
      };
    case "counter":
      return { ...shape, center: map(shape.center) };
    case "text":
      return { ...shape, origin: map(shape.origin) };
    case "draw":
      return { ...shape, points: shape.points.map(map) };
    case "highlight":
      return {
        ...shape,
        bands: shape.bands.map((band) => {
          const a = map({ x: band.left, y: band.top });
          const b = map({ x: band.right, y: band.bottom });
          return {
            bottom: Math.max(a.y, b.y),
            left: Math.min(a.x, b.x),
            right: Math.max(a.x, b.x),
            top: Math.min(a.y, b.y),
          };
        }),
        end: map(shape.end),
        start: map(shape.start),
      };
    case "magnify": {
      const loupe = map(shape.loupe);
      const edge = map({ x: shape.loupe.x + shape.size, y: shape.loupe.y });
      return {
        ...shape,
        end: map(shape.end),
        loupe,
        size: Math.abs(edge.x - loupe.x),
        start: map(shape.start),
      };
    }
    case "image": {
      const center = map(shape.center);
      const edge = map({ x: shape.center.x + shape.size, y: shape.center.y });
      return { ...shape, center, size: Math.abs(edge.x - center.x) };
    }
    case "redact":
    case "shape":
    case "spotlight":
      return { ...shape, end: map(shape.end), start: map(shape.start) };
  }
};
