// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CaptionLayout } from "../../../../bindings/CaptionLayout";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";

/** The layout options, which a template brings and the Layout tab sets by
 * hand: the twins of `CaptionLayout`'s fields in `screenwide_captions`. */
const LAYOUT_KEYS = [
  "accentSize",
  "allCaps",
  "lines",
  "maxWords",
  "position",
  "size",
] as const;

/** Every layout option `style` holds, as though each were set. */
export const layoutOf = (style: CaptionStyle): Required<CaptionLayout> => ({
  accentSize: style.accentSize,
  allCaps: style.allCaps,
  lines: style.lines,
  maxWords: style.maxWords,
  position: style.position,
  size: style.size,
});

/** `style` with the options `layout` sets in place of its own, as
 * `CaptionLayout::applied_to` does. */
export const withLayout = (
  layout: CaptionLayout,
  style: CaptionStyle,
): CaptionStyle => ({
  ...style,
  ...Object.fromEntries(
    LAYOUT_KEYS.flatMap((key) =>
      layout[key] === undefined ? [] : [[key, layout[key]]],
    ),
  ),
});

/** Whether `a` and `b` set the same options to the same values. */
export const isSameLayout = (a: CaptionLayout, b: CaptionLayout) =>
  LAYOUT_KEYS.every((key) => a[key] === b[key]);

export const isEmptyLayout = (layout: CaptionLayout) =>
  isSameLayout(layout, {});
