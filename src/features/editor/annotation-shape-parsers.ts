// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Each kind's stored shape, read back from a document: the shape as it can be
 * drawn, or null where the compositor could not place it. `ANNOTATION_KINDS`
 * gives each kind its reader from here.
 */

import {
  annotationBox,
  annotationPoint,
  annotationSeed,
  highlightBand,
  highlightTone,
  textPointer,
} from "./annotation-shape-parts";

import type {
  AnnotationArrow,
  AnnotationCounter,
  AnnotationHighlight,
  AnnotationShape,
  AnnotationText,
} from "./annotations";

type Shape<Kind extends AnnotationShape["kind"]> = Extract<
  AnnotationShape,
  { kind: Kind }
>;

export const arrowShape = (value: unknown): Shape<"arrow"> | null => {
  const shape = (value ?? {}) as Partial<AnnotationArrow>;
  const control = annotationPoint(shape.control);
  const end = annotationPoint(shape.end);
  const start = annotationPoint(shape.start);
  return control && end && start
    ? { control, end, kind: "arrow", start }
    : null;
};

export const counterShape = (value: unknown): Shape<"counter"> | null => {
  const shape = (value ?? {}) as Partial<AnnotationCounter>;
  const center = annotationPoint(shape.center);
  const number = shape.value;
  return center &&
    typeof shape.angle === "number" &&
    Number.isFinite(shape.angle) &&
    typeof number === "number" &&
    Number.isInteger(number) &&
    number >= 1
    ? { angle: shape.angle, center, kind: "counter", value: number }
    : null;
};

export const highlightShape = (value: unknown): Shape<"highlight"> | null => {
  const shape = (value ?? {}) as Partial<AnnotationHighlight>;
  const start = annotationPoint(shape.start);
  const end = annotationPoint(shape.end);
  const seed = annotationSeed(shape.seed);
  const bands = Array.isArray(shape.bands)
    ? shape.bands.map(highlightBand)
    : [];
  return start && end && seed !== null && bands.every((band) => band !== null)
    ? {
        bands,
        end,
        kind: "highlight",
        seed,
        start,
        tone: highlightTone(shape.tone),
      }
    : null;
};

export const redactShape = (value: unknown): Shape<"redact"> | null => {
  const box = annotationBox(value);
  return box && { ...box, kind: "redact" };
};

export const outlineShape = (value: unknown): Shape<"shape"> | null => {
  const box = annotationBox(value);
  return box && { ...box, kind: "shape" };
};

export const spotlightShape = (value: unknown): Shape<"spotlight"> | null => {
  const shape = (value ?? {}) as Record<string, unknown>;
  const start = annotationPoint(shape.start);
  const end = annotationPoint(shape.end);
  return start && end ? { end, kind: "spotlight", start } : null;
};

export const textShape = (value: unknown): Shape<"text"> | null => {
  const shape = (value ?? {}) as Partial<AnnotationText>;
  const origin = annotationPoint(shape.origin);
  return origin && typeof shape.text === "string"
    ? {
        kind: "text",
        origin,
        pointer: textPointer(shape.pointer),
        text: shape.text,
      }
    : null;
};
