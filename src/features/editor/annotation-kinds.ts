// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * What each kind of annotation is and can do, one row per kind: how it is read
 * from a document, how long it takes to arrive, what the lane calls it and
 * which controls the panel offers it.
 */

import { ANNOTATION_SIZES } from "../../components/shared/annotation-style/widths";

import {
  annotationPoint,
  highlightBand,
  highlightTone,
  textPointer,
} from "./annotation-shape-parts";

import type {
  Annotation,
  AnnotationArrow,
  AnnotationCounter,
  AnnotationHighlight,
  AnnotationRedact,
  AnnotationShape,
  AnnotationText,
} from "./annotations";
import type { AnnotationKind } from "../../components/shared/annotation-style/types";

/**
 * How long an animated annotation takes to arrive, in source milliseconds. The
 * twins of `REVEAL_DRAW_IN_MS`, `COUNTER_REVEAL_IN_MS` and `TEXT_REVEAL_IN_MS`
 * in `src-tauri/src/editor/annotations/reveal.rs`, `counter/reveal.rs` and
 * `text/reveal.rs`, which Rust tests hold to these lines; a reveal may shorten
 * its phase for a short clip but never lengthens it, so an annotation placed
 * this long before the playhead is always whole by the time the playhead is
 * reached. A text box's is its box's arrival and then its pointer's.
 */
export const ANNOTATION_DRAW_IN_MS = 1000;
const ANNOTATION_COUNTER_DRAW_IN_MS = 320;
const ANNOTATION_TEXT_DRAW_IN_MS = 600;

/**
 * What one kind of annotation is and can do: how it is read from a document,
 * how long it takes to arrive, what it is called, and which controls it
 * answers to.
 *
 * One row per kind rather than a condition at each control, so a tool added
 * later states its own answers in one place and the panel, the lane and the
 * document reader all follow. The sizes come from the shared table the live
 * overlay's controls read, so a kind is offered the same sizes wherever it is
 * drawn.
 */
type AnnotationKindRow<Shape extends AnnotationShape> =
  (typeof ANNOTATION_SIZES)[AnnotationKind] & {
    /** Whether it draws itself in over its clip, which is what the panel's
     * Animate switch offers. */
    animates: boolean;
    drawInMs: number;
    /** Whether it lines up lines of text. */
    hasAlign: boolean;
    /** Whether it is aimed by an angle of its own, rather than by its ends. */
    hasAngle: boolean;
    /** Whether it fits itself to what is under it, or can be laid by hand
     * over a box instead. */
    hasFit: boolean;
    /** Whether it can be drawn by hand, and drawn again differently. */
    hasHandDrawn: boolean;
    /** Whether it carries heads to choose between. */
    hasHead: boolean;
    /** Whether it hides what is under it, and so offers the redaction modes. */
    hasRedaction: boolean;
    /** Whether its size is its own to choose. A highlight's bands are as tall
     * as the lines it covers, and a box is laid with its one marker. */
    hasSize: boolean;
    /** What the timeline lane calls one, `index` being its place in the lane. */
    laneLabel: (shape: Shape, index: number) => string;
    /** The shape as stored, or null where the compositor could not place it. */
    parseShape: (value: unknown) => Shape | null;
    /** Whether it can be turned round end for end. */
    reversible: boolean;
    /** Whether a fresh one arrives without drawing itself in, whatever the
     * last one did: a redaction would show what it hides until it had. */
    startsStill: boolean;
  };

export const ANNOTATION_KINDS: {
  [Kind in AnnotationKind]: AnnotationKindRow<
    Extract<AnnotationShape, { kind: Kind }>
  >;
} = {
  arrow: {
    ...ANNOTATION_SIZES.arrow,
    animates: true,
    drawInMs: ANNOTATION_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: true,
    hasRedaction: false,
    hasSize: true,
    // An arrow has no name of its own, so it is called by its place in the
    // lane.
    laneLabel: (_shape, index) => `Arrow ${String(index + 1)}`,
    parseShape: (value) => {
      const shape = (value ?? {}) as Partial<AnnotationArrow>;
      const control = annotationPoint(shape.control);
      const end = annotationPoint(shape.end);
      const start = annotationPoint(shape.start);
      return control && end && start
        ? { control, end, kind: "arrow", start }
        : null;
    },
    reversible: true,
    startsStill: false,
  },
  counter: {
    ...ANNOTATION_SIZES.counter,
    animates: true,
    drawInMs: ANNOTATION_COUNTER_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: true,
    hasFit: false,
    hasHandDrawn: false,
    // A counter is a disc with a number in it, which leaves it no head to
    // choose and nothing to reverse.
    hasHead: false,
    hasRedaction: false,
    hasSize: true,
    laneLabel: (shape) => `Counter ${String(shape.value)}`,
    parseShape: (value) => {
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
    },
    reversible: false,
    startsStill: false,
  },
  highlight: {
    ...ANNOTATION_SIZES.highlight,
    // Drawn along its bands the way an arrow is along its path.
    animates: true,
    drawInMs: ANNOTATION_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasFit: true,
    hasHandDrawn: true,
    hasHead: false,
    hasRedaction: false,
    hasSize: false,
    laneLabel: (_shape, index) => `Highlight ${String(index + 1)}`,
    parseShape: (value) => {
      const shape = (value ?? {}) as Partial<AnnotationHighlight>;
      const start = annotationPoint(shape.start);
      const end = annotationPoint(shape.end);
      const seed = shape.seed;
      const bands = Array.isArray(shape.bands)
        ? shape.bands.map(highlightBand)
        : [];
      return start &&
        end &&
        typeof seed === "number" &&
        Number.isInteger(seed) &&
        seed >= 0 &&
        seed <= 0xffffffff &&
        bands.every((band) => band !== null)
        ? {
            bands,
            end,
            kind: "highlight",
            seed,
            start,
            tone: highlightTone(shape.tone),
          }
        : null;
    },
    reversible: false,
    startsStill: false,
  },
  redact: {
    ...ANNOTATION_SIZES.redact,
    // A redaction ramps in on a counter's timing and never leaves. It starts
    // without the ramp, since a box arriving shows what it hides until it
    // has.
    animates: true,
    drawInMs: ANNOTATION_COUNTER_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasRedaction: true,
    hasSize: true,
    laneLabel: (_shape, index) => `Redaction ${String(index + 1)}`,
    parseShape: (value) => {
      const shape = (value ?? {}) as Partial<AnnotationRedact>;
      const start = annotationPoint(shape.start);
      const end = annotationPoint(shape.end);
      const seed = shape.seed;
      return start &&
        end &&
        typeof seed === "number" &&
        Number.isInteger(seed) &&
        seed >= 0 &&
        seed <= 0xffffffff
        ? { end, kind: "redact", seed, start }
        : null;
    },
    reversible: false,
    startsStill: true,
  },
  text: {
    ...ANNOTATION_SIZES.text,
    animates: true,
    // The box grows into place the way a counter does, and then its pointer
    // draws out of it.
    drawInMs: ANNOTATION_TEXT_DRAW_IN_MS,
    hasAlign: true,
    hasAngle: false,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasRedaction: false,
    hasSize: true,
    // A box is called by what it says: its first line, which the lane
    // truncates to the room it has, or by its place when that line is empty.
    laneLabel: (shape, index) => {
      const line = shape.text.split("\n", 1)[0].trim();
      return line === "" ? `Text ${String(index + 1)}` : line;
    },
    parseShape: (value) => {
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
    },
    reversible: false,
    startsStill: false,
  },
};

/** Whether `value` names a kind this build knows how to draw. */
export const isAnnotationKind = (value: unknown): value is AnnotationKind =>
  typeof value === "string" && value in ANNOTATION_KINDS;

/** How long `annotation` takes to arrive: a counter grows into place far
 * quicker than an arrow draws itself. A redaction that does not animate is
 * whole from its clip's first frame, so its clip starts where it is placed
 * rather than reaching back. */
export const annotationDrawInMs = (annotation: Annotation) =>
  annotation.shape.kind === "redact" && !annotation.animated
    ? 0
    : ANNOTATION_KINDS[annotation.shape.kind].drawInMs;

/** What the timeline lane calls `annotation`, `index` being its place there. */
export const annotationLaneLabel = (annotation: Annotation, index: number) => {
  const { shape } = annotation;
  // The row is looked up by the shape's own kind, so the two always agree;
  // TypeScript cannot carry that correlation through the lookup.
  const label = ANNOTATION_KINDS[shape.kind].laneLabel as (
    shape: AnnotationShape,
    index: number,
  ) => string;
  return label(shape, index);
};
