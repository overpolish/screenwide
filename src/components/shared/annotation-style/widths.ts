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

/** The stroke a fresh arrow is drawn with. The twin of `NEW_ARROW_WIDTH`. */
export const DEFAULT_ANNOTATION_WIDTH = 8;

/** Where `width` sits in the preset list: the nearest preset to it, so a
 * width from an older document still lands the knob somewhere sensible. */
export const annotationWidthIndex = (width: number) => {
  if (!Number.isFinite(width))
    return ANNOTATION_WIDTHS.indexOf(DEFAULT_ANNOTATION_WIDTH);
  let nearest = 0;
  for (let index = 1; index < ANNOTATION_WIDTHS.length; index++) {
    if (
      Math.abs(ANNOTATION_WIDTHS[index] - width) <
      Math.abs(ANNOTATION_WIDTHS[nearest] - width)
    )
      nearest = index;
  }
  return nearest;
};

/** The preset at `index`, clamped to the list the control runs over. */
export const annotationWidthAt = (index: number) =>
  ANNOTATION_WIDTHS[
    Math.min(
      ANNOTATION_WIDTHS.length - 1,
      Math.max(0, Math.round(Number.isFinite(index) ? index : 0)),
    )
  ];
