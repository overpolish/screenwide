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

/** The options in `layout` the template does not already give: one set to
 * the template's own value is not kept, so it follows the next template
 * chosen as an untouched one does. Without the template there is nothing to
 * compare with, so `layout` stays as it is. */
export const handLayout = (
  layout: TextLayout,
  template: TextStyle | undefined,
): TextLayout =>
  template
    ? Object.fromEntries(
        LAYOUT_KEYS.flatMap((key) =>
          layout[key] === undefined || layout[key] === template[key]
            ? []
            : [[key, layout[key]]],
        ),
      )
    : layout;

/** Whether `a` and `b` set the same options to the same values. */
export const isSameLayout = (a: TextLayout, b: TextLayout) =>
  LAYOUT_KEYS.every((key) => a[key] === b[key]);

export const isEmptyLayout = (layout: TextLayout) => isSameLayout(layout, {});
