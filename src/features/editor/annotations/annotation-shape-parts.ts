// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The pieces a stored annotation's shape is read from, each held to what the
 * compositor can draw: a point, a seed, a box's corners, a text's look and
 * its control points, a highlight's band and the page it was read from.
 */

import type {
  AnnotationHighlight,
  AnnotationPoint,
  AnnotationText,
  HighlightBand,
  TextControlPoint,
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

/** A stored seed, or null where it is not a 32-bit unsigned integer. */
export const annotationSeed = (value: unknown): number | null =>
  typeof value === "number" &&
  Number.isInteger(value) &&
  value >= 0 &&
  value <= 0xffffffff
    ? value
    : null;

/** A stored box's two corners and its seed - a redaction's or a shape's - or
 * null where any of them cannot be read. */
export const annotationBox = (value: unknown) => {
  const box = (value ?? {}) as Record<string, unknown>;
  const start = annotationPoint(box.start);
  const end = annotationPoint(box.end);
  const seed = annotationSeed(box.seed);
  return start && end && seed !== null ? { end, seed, start } : null;
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

/** A stored look, kept as it was written where it names its template: Rust
 * reads it and holds it to what a style can be. A text with none takes the
 * Label look there. */
export const textLook = (value: unknown): Pick<AnnotationText, "look"> => {
  if (
    typeof value !== "object" ||
    value === null ||
    !("template" in value) ||
    typeof value.template !== "string"
  )
    return {};
  // Rust wrote it, and Rust reads it back to its own rules.
  const look = value as NonNullable<AnnotationText["look"]>;
  return { look };
};

/** A pair of finite numbers, or null. */
const finitePair = (value: unknown): [number, number] | null =>
  Array.isArray(value) &&
  value.length === 2 &&
  value.every((entry) => typeof entry === "number" && Number.isFinite(entry))
    ? [value[0] as number, value[1] as number]
    : null;

/** A stored control point, or null where it cannot be read. */
const textControlPoint = (value: unknown): TextControlPoint | null => {
  if (typeof value !== "object" || value === null || !("space" in value))
    return null;
  if (value.space === "picture") {
    const at = annotationPoint(value);
    return at ? { space: "picture", ...at } : null;
  }
  const along = "along" in value ? finitePair(value.along) : null;
  const reach = "reach" in value ? finitePair(value.reach) : null;
  return value.space === "box" && along && reach
    ? {
        along: [
          Math.min(1, Math.max(-1, along[0])),
          Math.min(1, Math.max(-1, along[1])),
        ],
        reach: [Math.max(0, reach[0]), Math.max(0, reach[1])],
        space: "box",
      }
    : null;
};

/** The stored control points that can be read; the rest start again where
 * the look puts them. */
export const textControls = (
  value: unknown,
): Pick<AnnotationText, "controls"> => {
  if (typeof value !== "object" || value === null) return {};
  const controls = Object.fromEntries(
    Object.entries(value).flatMap(([id, point]) => {
      const read = textControlPoint(point);
      return read ? [[id, read] as const] : [];
    }),
  );
  return Object.keys(controls).length > 0 ? { controls } : {};
};

/** How far past its box a text's tail reaches, in ems, where its look draws
 * one; a tail put on the picture counts as reaching `pictureReach`. The twin
 * of the text arm of `pace::path_ms` in Rust. */
export const textTailReach = (
  shape: AnnotationText,
  pictureReach: number,
): number => {
  const look = shape.look;
  if (!look || look.background !== "box") return 0;
  // A tail is the one kind of element there is, and a look draws one at most.
  const control = look.controls.find((control) =>
    look.elements.some((element) => element.to === control.id),
  );
  if (!control) return 0;
  const placed = shape.controls?.[control.id];
  if (placed?.space === "picture" && control.space === "picture")
    return pictureReach;
  const reach =
    placed?.space === "box" && control.space === "box"
      ? placed.reach
      : control.default.reach;
  return Math.hypot(reach[0], reach[1]);
};
