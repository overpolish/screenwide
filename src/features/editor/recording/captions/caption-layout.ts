// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TextLayout } from "../../../../bindings/TextLayout";
import type { TextStyle } from "../../../../bindings/TextStyle";

/** The layout options, which a template brings and the Layout tab sets by
 * hand: the twins of `TextLayout`'s fields in `screenwide_text`. */
const LAYOUT_KEYS = [
  "accentSize",
  "allCaps",
  "lines",
  "maxWords",
  "position",
  "size",
] as const;

/** Every layout option `style` holds, as though each were set. */
export const layoutOf = (style: TextStyle): Required<TextLayout> => ({
  accentSize: style.accentSize,
  allCaps: style.allCaps,
  lines: style.lines,
  maxWords: style.maxWords,
  position: style.position,
  size: style.size,
});

/** `style` with the options `layout` sets in place of its own, as
 * `TextLayout::applied_to` does. */
export const withLayout = (
  layout: TextLayout,
  style: TextStyle,
): TextStyle => ({
  ...style,
  ...Object.fromEntries(
    LAYOUT_KEYS.flatMap((key) =>
      layout[key] === undefined ? [] : [[key, layout[key]]],
    ),
  ),
});

/** Whether `a` and `b` set the same options to the same values. */
export const isSameLayout = (a: TextLayout, b: TextLayout) =>
  LAYOUT_KEYS.every((key) => a[key] === b[key]);

export const isEmptyLayout = (layout: TextLayout) => isSameLayout(layout, {});
