// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The strokes the width controls offer, in output pixels.
 *
 * The steps are not uniform - the jump from hairline to visible matters more
 * than the one from thick to thicker - so a control runs over the index of
 * this list rather than over the width itself. The twin of `MIN_WIDTH` and
 * `MAX_WIDTH` in `src-tauri/src/annotate/settings.rs`, which refuses a live
 * stroke outside these ends.
 */
export const ANNOTATION_WIDTHS = [8, 12, 16, 24, 32, 48];

/**
 * The disc diameters a counter offers, in output pixels. Three sizes far
 * enough apart to be worth choosing between: a handful of pixels either way
 * is no choice at all. The twin of `COUNTER_SIZES` in
 * `src-tauri/src/editor/annotations/counter.rs`.
 */
export const ANNOTATION_COUNTER_SIZES = [56, 96, 160];

/** The stroke a fresh arrow is drawn with. The twin of `NEW_ARROW_WIDTH`. */
export const DEFAULT_ANNOTATION_WIDTH = 8;

/** The disc a fresh counter is drawn at: the smallest of the three. The twin
 * of `NEW_COUNTER_WIDTH`. */
export const DEFAULT_ANNOTATION_COUNTER_SIZE = 56;

/** The sizes an annotation of this kind is offered, and the one a fresh
 * annotation takes. */
export const annotationSizes = (kind: "arrow" | "counter") =>
  kind === "counter" ? ANNOTATION_COUNTER_SIZES : ANNOTATION_WIDTHS;

export const defaultAnnotationSize = (kind: "arrow" | "counter") =>
  kind === "counter"
    ? DEFAULT_ANNOTATION_COUNTER_SIZE
    : DEFAULT_ANNOTATION_WIDTH;

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
