// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { firstAnnotationDress } from "./annotation-defaults";
import { Annotation, AnnotationShape, AnnotationStyle } from "./annotations";
import { relaidHighlight } from "./highlight-strokes";

/** A fresh seed from the browser's secure generator, as a fresh redaction's
 * comes from the system's. */
export const freshSeed = () => crypto.getRandomValues(new Uint32Array(1))[0];

/**
 * The shape turned round, or null for one that cannot be. An arrow's head
 * rides its end point, so swapping the ends points it the other way, and the
 * bend keeps its control, so the curve is the mirror of itself rather than a
 * different one. An image is mirrored instead. A counter has no ends to
 * swap; the panel does not offer the row for one.
 */
export const reversedShape = (shape: AnnotationShape) =>
  shape.kind === "arrow"
    ? { ...shape, end: shape.start, start: shape.end }
    : shape.kind === "image"
      ? { ...shape, flip: !shape.flip }
      : null;

/**
 * The shape drawn again from `seed`, or null for one with nothing seeded:
 * a redaction's blocks laid out again, a hand-drawn highlight's or shape's
 * stroke drawn again, or a swaying image swaying another way. The blocks
 * carry nothing of the picture's layout whatever the seed.
 */
export const shuffledShape = (shape: AnnotationShape, seed: number) =>
  shape.kind === "redact" ||
  shape.kind === "highlight" ||
  shape.kind === "shape"
    ? { ...shape, seed }
    : shape.kind === "image" && shape.sway !== undefined
      ? { ...shape, sway: seed }
      : null;

/** The shape `annotation` takes dressed in `next`: a highlight switched to
 * manual is laid out again, and turned hand-drawn, a stroke is drawn from a
 * fresh seed, so each time it wobbles its own way. */
export const dressedShape = (annotation: Annotation, next: AnnotationStyle) => {
  const relaid = relaidHighlight(annotation, next) ?? annotation.shape;
  return next.handDrawn && !annotation.style.handDrawn
    ? (shuffledShape(relaid, freshSeed()) ?? relaid)
    : relaid;
};

/**
 * An image that sways from `seed`, or with none, one that stands still and
 * keeps nothing of how it swayed. Null for anything but an image.
 */
export const swayedShape = (shape: AnnotationShape, seed: number | null) => {
  if (shape.kind !== "image") return null;
  const { sway: _sway, ...still } = shape;
  return seed === null ? still : { ...still, sway: seed };
};

/**
 * The annotation in the other of a focus's two looks, blurred and dimmed,
 * or null where it is neither or already in `kind`. It keeps its box and its
 * corners and takes the other look's own first dress for the rest, its
 * shade, its edge and its blur.
 */
export const relookedFocus = (
  annotation: Annotation,
  kind: "focus" | "spotlight",
): Annotation | null => {
  const { shape, style } = annotation;
  if (
    shape.kind === kind ||
    (shape.kind !== "focus" && shape.kind !== "spotlight")
  )
    return null;
  return {
    ...annotation,
    shape: { end: shape.end, kind, start: shape.start },
    style: { ...firstAnnotationDress(kind), radius: style.radius },
  };
};
