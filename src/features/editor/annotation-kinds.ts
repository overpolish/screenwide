// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * What each kind of annotation is and can do, one row per kind: how it is read
 * from a document, how long it takes to arrive, what the lane calls it and
 * which controls the panel offers it.
 */

import { ANNOTATION_SIZES } from "../../components/shared/annotation-style/widths";

import {
  arrowShape,
  counterShape,
  drawShape,
  highlightShape,
  magnifyShape,
  outlineShape,
  redactShape,
  spotlightShape,
  textShape,
} from "./annotation-shape-parsers";

import type { AnnotationKindRow } from "./annotation-kind-row";
import type { Annotation, AnnotationShape } from "./annotations";
import type { AnnotationKind } from "../../components/shared/annotation-style/types";

/**
 * How long an animated annotation takes to arrive where its clip names no
 * pace, in source milliseconds. The twins of `REVEAL_DRAW_IN_MS`,
 * `COUNTER_REVEAL_IN_MS`, `TEXT_REVEAL_IN_MS`, `SPOTLIGHT_FADE_IN_MS` and
 * `MAGNIFY_REVEAL_IN_MS` in `src-tauri/src/editor/annotations/reveal.rs`,
 * `counter/reveal.rs`, `text/reveal.rs`, `spotlight/reveal.rs` and
 * `magnify/reveal.rs`, which Rust tests hold to these
 * lines. A clip the editor paces arrives over its path's own time instead, as
 * `annotation-pace.ts` works it out. A text box's is its box's arrival and
 * then its pointer's, and a spotlight's is its shade deepening.
 */
export const ANNOTATION_DRAW_IN_MS = 1000;
const ANNOTATION_COUNTER_DRAW_IN_MS = 320;
const ANNOTATION_TEXT_DRAW_IN_MS = 600;
const ANNOTATION_SPOTLIGHT_DRAW_IN_MS = 400;
const ANNOTATION_MAGNIFY_DRAW_IN_MS = 450;

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
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: true,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    // An arrow has no name of its own, so it is called by its place in the
    // lane.
    laneLabel: (_shape, index) => `Arrow ${String(index + 1)}`,
    parseShape: arrowShape,
    reversible: true,
    startsStill: false,
  },
  counter: {
    ...ANNOTATION_SIZES.counter,
    animates: true,
    drawInMs: ANNOTATION_COUNTER_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: true,
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: false,
    // A counter is a disc with a number in it, which leaves it no head to
    // choose and nothing to reverse.
    hasHead: false,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    laneLabel: (shape) => `Counter ${String(shape.value)}`,
    parseShape: counterShape,
    reversible: false,
    startsStill: false,
  },
  draw: {
    ...ANNOTATION_SIZES.draw,
    // Drawn along its line the way an arrow is along its path.
    animates: true,
    drawInMs: ANNOTATION_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    // A stroke already is one.
    hasHandDrawn: false,
    hasHead: false,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    laneLabel: (_shape, index) => `Drawing ${String(index + 1)}`,
    parseShape: drawShape,
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
    hasBlur: false,
    hasColor: true,
    hasFit: true,
    hasHandDrawn: true,
    hasHead: false,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: false,
    hasSoftness: false,
    laneLabel: (_shape, index) => `Highlight ${String(index + 1)}`,
    parseShape: highlightShape,
    reversible: false,
    startsStill: false,
  },
  magnify: {
    ...ANNOTATION_SIZES.magnify,
    // The loupe comes out of its zoom area and grows to its place as it
    // arrives, and goes back into it as it leaves. Its rim is always an
    // arrow's pen, and its zoom is how big its loupe is drawn.
    animates: true,
    drawInMs: ANNOTATION_MAGNIFY_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasRadius: true,
    hasRedaction: false,
    hasShadow: true,
    hasSize: false,
    hasSoftness: false,
    laneLabel: (_shape, index) => `Magnifier ${String(index + 1)}`,
    parseShape: magnifyShape,
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
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasRadius: true,
    hasRedaction: true,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    laneLabel: (_shape, index) => `Redaction ${String(index + 1)}`,
    parseShape: redactShape,
    reversible: false,
    startsStill: true,
  },
  shape: {
    ...ANNOTATION_SIZES.shape,
    // Drawn round its outline the way an arrow is along its path.
    animates: true,
    drawInMs: ANNOTATION_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: true,
    hasHead: false,
    hasRadius: true,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    laneLabel: (_shape, index) => `Shape ${String(index + 1)}`,
    parseShape: outlineShape,
    reversible: false,
    startsStill: false,
  },
  spotlight: {
    ...ANNOTATION_SIZES.spotlight,
    // The shade deepens round it as it arrives and lifts as it leaves. It is
    // no colour and no stroke: only its box, its corners, its fade and
    // whether what is outside it blurs.
    animates: true,
    drawInMs: ANNOTATION_SPOTLIGHT_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: true,
    hasColor: false,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasRadius: true,
    hasRedaction: false,
    hasShadow: false,
    hasSize: false,
    hasSoftness: true,
    laneLabel: (_shape, index) => `Spotlight ${String(index + 1)}`,
    parseShape: spotlightShape,
    reversible: false,
    startsStill: false,
  },
  text: {
    ...ANNOTATION_SIZES.text,
    animates: true,
    // The box grows into place the way a counter does, and then its pointer
    // draws out of it.
    drawInMs: ANNOTATION_TEXT_DRAW_IN_MS,
    hasAlign: true,
    hasAngle: false,
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    // A box is called by what it says: its first line, which the lane
    // truncates to the room it has, or by its place when that line is empty.
    laneLabel: (shape, index) => {
      const line = shape.text.split("\n", 1)[0].trim();
      return line === "" ? `Text ${String(index + 1)}` : line;
    },
    parseShape: textShape,
    reversible: false,
    startsStill: false,
  },
};

/** Whether `value` names a kind this build knows how to draw. */
export const isAnnotationKind = (value: unknown): value is AnnotationKind =>
  typeof value === "string" && value in ANNOTATION_KINDS;

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
