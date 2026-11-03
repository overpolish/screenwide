// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * What each kind of annotation is and can do, one row per kind: how it is read
 * from a document, how long it takes to arrive, what the lane calls it and
 * which controls the panel offers it. `annotation-kind-lookup.ts` reads the
 * table for one stored kind or one annotation. The kinds held by a box have
 * their rows in `annotation-box-kinds.ts`, and every kind's arrival time is
 * in `annotation-draw-in.ts`.
 */

import { ANNOTATION_SIZES } from "../../annotations/widths";

import { BOX_KIND_ROWS } from "./annotation-box-kinds";
import {
  ANNOTATION_COUNTER_DRAW_IN_MS,
  ANNOTATION_DRAW_IN_MS,
  ANNOTATION_MAGNIFY_DRAW_IN_MS,
  ANNOTATION_TEXT_DRAW_IN_MS,
} from "./annotation-draw-in";
import {
  arrowShape,
  counterShape,
  drawShape,
  highlightShape,
  magnifyShape,
  imageShape,
  textShape,
} from "./annotation-shape-parsers";

import type { AnnotationKindRow } from "./annotation-kind-row";
import type { AnnotationShape } from "./annotations";
import type { AnnotationKind } from "../../annotations/types";

export const ANNOTATION_KINDS: {
  [Kind in AnnotationKind]: AnnotationKindRow<
    Extract<AnnotationShape, { kind: Kind }>
  >;
} = {
  ...BOX_KIND_ROWS,
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
    hasInk: false,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
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
    hasInk: false,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
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
    hasInk: false,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
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
    hasInk: true,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: false,
    hasSoftness: false,
    parseShape: highlightShape,
    reversible: false,
    startsStill: false,
  },
  image: {
    ...ANNOTATION_SIZES.image,
    // It pops in and out the way a counter does. It shows a picture of its
    // own rather than a colour, is sized by its grips, turned by its own grip
    // or the angle control, and mirrored rather than turned end for end.
    animates: true,
    drawInMs: ANNOTATION_COUNTER_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: false,
    hasColor: false,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasInk: false,
    hasRadius: true,
    hasRedaction: false,
    hasShadow: true,
    hasSize: false,
    hasSoftness: false,
    parseShape: imageShape,
    reversible: true,
    startsStill: false,
  },
  magnify: {
    ...ANNOTATION_SIZES.magnify,
    // The lens grows from a smaller, flatter glass to its size and zoom as
    // it arrives, and back as it leaves. Its rim is always an arrow's pen,
    // and its zoom is its shape's own.
    animates: true,
    drawInMs: ANNOTATION_MAGNIFY_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasInk: false,
    hasRadius: true,
    hasRedaction: false,
    hasShadow: true,
    hasSize: false,
    hasSoftness: false,
    parseShape: magnifyShape,
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
    hasInk: false,
    hasRadius: false,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    parseShape: textShape,
    reversible: false,
    startsStill: false,
  },
};
