// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationSizeMeasure } from "./annotation-text";
import type { AnnotationKind, AnnotationRedaction } from "./types";

/**
 * The strokes the width controls offer, in points: drawn at the capture's
 * scale, so a preset weighs the same on a 2x capture as on a 1x one.
 *
 * The steps are not uniform - the jump from hairline to visible matters more
 * than the one from thick to thicker - so a control runs over the index of
 * this list rather than over the width itself. The twin of `MIN_WIDTH` and
 * `MAX_WIDTH` in `src-tauri/src/annotate/settings.rs`, which refuses a live
 * stroke outside these ends.
 */
export const ANNOTATION_WIDTHS = [8, 12, 16, 24];

/**
 * The disc diameters a counter offers, in points. The twin of `COUNTER_SIZES`
 * in `src-tauri/src/editor/annotations/counter/model.rs`.
 */
export const ANNOTATION_COUNTER_SIZES = [28, 36, 48, 64, 80];

/** The stroke a fresh arrow or shape is drawn with. The twin of
 * `NEW_ARROW_WIDTH` and `NEW_SHAPE_WIDTH`. */
export const DEFAULT_ANNOTATION_WIDTH = 8;

/** The disc a fresh counter is drawn at: the smallest. The twin of
 * `NEW_COUNTER_WIDTH`. */
export const DEFAULT_ANNOTATION_COUNTER_SIZE = 28;

/**
 * The type sizes a text box offers, in points: two small steps for labels,
 * then steps that grow by a ratio so the large ones are big enough to title a
 * screen. The twin of `TEXT_SIZES` in
 * `src-tauri/src/editor/annotations/text/model.rs`.
 */
const ANNOTATION_TEXT_SIZES = [10, 14, 24, 40, 64];

/** The type size a fresh text box is set at: the third step. The twin of
 * `NEW_TEXT_WIDTH`. */
const DEFAULT_ANNOTATION_TEXT_SIZE = 24;

/**
 * The block sizes a securely pixelated redaction offers, in logical points of
 * the captured content, so a block hides as much on a 2x capture as on a 1x
 * one. The twin of `REDACT_BLOCK_SIZES` in
 * `src-tauri/src/editor/annotations/redact/model.rs`.
 */
const ANNOTATION_REDACT_SIZES = [4, 6, 8, 12, 16];

/**
 * The block sizes classic pixelation offers, in logical points: coarser than
 * the secure steps, since fine ordinary blocks read as the picture itself.
 * The first is the twin of `MIN_CLASSIC_BLOCK` in
 * `src-tauri/src/editor/annotations/redact/cells.rs`.
 */
const ANNOTATION_CLASSIC_SIZES = [8, 12, 16, 24, 32];

/** The block sizes a redaction drawn `redaction` offers: classic
 * pixelation's own coarser steps, or the kind's. */
export const redactionSizePresets = (
  redaction: AnnotationRedaction,
  sizes: number[],
) => (redaction === "pixelateClassic" ? ANNOTATION_CLASSIC_SIZES : sizes);

/** The marker every highlight is drawn with, in points: the stroke a box laid
 * by hand is covered in, and the band drawn where no text is found. The twin
 * of `NEW_HIGHLIGHT_WIDTH` in
 * `src-tauri/src/editor/annotations/highlight/model.rs`. */
const DEFAULT_ANNOTATION_HIGHLIGHT_SIZE = 12;

/** The block a fresh redaction pixelates with: the second step. The twin of
 * `NEW_REDACT_WIDTH`. */
const DEFAULT_ANNOTATION_REDACT_SIZE = 6;

/**
 * The strengths a blurred redaction offers, weakest first: each step blurs
 * wider, the same width in every box. The twin of `BLUR_DEVIATIONS` in
 * `src-tauri/src/editor/annotations/redact/cells.rs`, which sizes them.
 */
export const ANNOTATION_BLUR_STRENGTHS = [1, 2, 3, 4, 5];

/** The strength a fresh blur takes: the middle step. The twin of
 * `NEW_BLUR_STRENGTH`. */
export const DEFAULT_BLUR_STRENGTH = 3;

/** How far a soft spotlight's edge fades in from its box, as a percentage of
 * its shorter side; a hard one fades over none. The twin of Rust's
 * `NEW_SPOTLIGHT_SOFTNESS`. */
export const SOFT_SPOTLIGHT_EDGE = 10;

/**
 * How big each kind is drawn: the sizes its control offers, the one a fresh
 * annotation takes, and what the control measures. One row per kind, shared
 * because the editor's panel and the live overlay's toolbar offer the same
 * choice.
 */
export const ANNOTATION_SIZES: Record<
  AnnotationKind,
  { defaultSize: number; sizeMeasure: AnnotationSizeMeasure; sizes: number[] }
> = {
  arrow: {
    defaultSize: DEFAULT_ANNOTATION_WIDTH,
    sizeMeasure: "width",
    sizes: ANNOTATION_WIDTHS,
  },
  counter: {
    defaultSize: DEFAULT_ANNOTATION_COUNTER_SIZE,
    sizeMeasure: "size",
    sizes: ANNOTATION_COUNTER_SIZES,
  },
  draw: {
    defaultSize: DEFAULT_ANNOTATION_WIDTH,
    sizeMeasure: "width",
    sizes: ANNOTATION_WIDTHS,
  },
  highlight: {
    defaultSize: DEFAULT_ANNOTATION_HIGHLIGHT_SIZE,
    sizeMeasure: "height",
    sizes: [DEFAULT_ANNOTATION_HIGHLIGHT_SIZE],
  },
  // An image is sized by its grips and draws no stroke: it carries the
  // least width every clip must, the twin of `default_image_style`'s.
  image: {
    defaultSize: 1,
    sizeMeasure: "size",
    sizes: [1],
  },
  // A loupe's rim is always an arrow's pen, the twin of `NEW_MAGNIFY_WIDTH`,
  // and offers no size to choose.
  magnify: {
    defaultSize: DEFAULT_ANNOTATION_WIDTH,
    sizeMeasure: "width",
    sizes: [DEFAULT_ANNOTATION_WIDTH],
  },
  redact: {
    defaultSize: DEFAULT_ANNOTATION_REDACT_SIZE,
    sizeMeasure: "blockSize",
    sizes: ANNOTATION_REDACT_SIZES,
  },
  shape: {
    defaultSize: DEFAULT_ANNOTATION_WIDTH,
    sizeMeasure: "width",
    sizes: ANNOTATION_WIDTHS,
  },
  // A spotlight draws no stroke: it carries the least width every clip must,
  // the twin of `default_spotlight_style`'s, and offers no size to choose.
  spotlight: {
    defaultSize: 1,
    sizeMeasure: "width",
    sizes: [1],
  },
  text: {
    defaultSize: DEFAULT_ANNOTATION_TEXT_SIZE,
    sizeMeasure: "size",
    sizes: ANNOTATION_TEXT_SIZES,
  },
};

/** The sizes an annotation of this kind is offered, and the one a fresh
 * annotation takes. */
export const annotationSizes = (kind: AnnotationKind) =>
  ANNOTATION_SIZES[kind].sizes;

export const defaultAnnotationSize = (kind: AnnotationKind) =>
  ANNOTATION_SIZES[kind].defaultSize;

/** Where `width` sits in `presets`: the nearest one to it, so a width from an
 * older document still lands the knob somewhere sensible. */
export const annotationWidthIndex = (
  width: number,
  presets: number[] = ANNOTATION_WIDTHS,
) => {
  if (!Number.isFinite(width))
    return Math.max(presets.indexOf(DEFAULT_ANNOTATION_WIDTH), 0);
  let nearest = 0;
  for (let index = 1; index < presets.length; index++) {
    if (Math.abs(presets[index] - width) < Math.abs(presets[nearest] - width))
      nearest = index;
  }
  return nearest;
};

/** The preset at `index`, clamped to the list the control runs over. */
export const annotationWidthAt = (
  index: number,
  presets: number[] = ANNOTATION_WIDTHS,
) =>
  presets[
    Math.min(
      presets.length - 1,
      Math.max(0, Math.round(Number.isFinite(index) ? index : 0)),
    )
  ];
