// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowUpRight,
  createLucideIcon,
  EyeOff,
  Focus,
  Flashlight,
  Highlighter,
  Image,
  Pencil,
  Square,
  Type,
} from "lucide-react";

/**
 * The glyphs the annotation tools are drawn with.
 *
 * The same tools are picked up in three places - the live overlay's toolbar,
 * the screenshot workspace and the recording workspace - and a tool that
 * looked different depending on where it was taken up would read as a
 * different tool. They are named here so a change reaches all three.
 */
export const ArrowToolIcon = ArrowUpRight;
/** Lucide's `MapPinPlusInside` with a 1 in place of its plus: a counter is
 * a numbered pin, and the plus read as adding a place. The 1 is Lucide's own
 * numeral from `Heading1` (`m17 12 3-2v8`) at three quarters, as tall as the
 * plus was, its stem just right of the disc's centre so the flag balances
 * it; centred by its box it leans right. */
export const CounterToolIcon = createLucideIcon({
  name: "counter",
  node: [
    [
      "path",
      {
        d: "M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0",
        key: "pin",
      },
    ],
    ["path", { d: "m10 8.5 2.25-1.5v6", key: "one" }],
  ],
  size: 24,
});
export const TextToolIcon = Type;
export const RedactToolIcon = EyeOff;
export const HighlightToolIcon = Highlighter;
export const ShapeToolIcon = Square;
export const SpotlightToolIcon = Flashlight;
export const FocusToolIcon = Focus;
export const DrawToolIcon = Pencil;
/** What a magnifier draws: an area, and a lens over its corner showing that
 * corner larger. Built to Lucide's design principles
 * (https://lucide.dev/contribute/icons/design-principles): the lens is the
 * library's own badge circle (as `calendar-clock`'s), the area a rounded
 * square of the 2-unit radius its size takes, stopping 2 units clear of the
 * lens as Lucide's badged icons do (10 from its centre), with round caps.
 * Inside the lens the corner is drawn again larger, at (17, 17) with the
 * same radius, its sides running out to the lens as `globe`'s lines meet
 * its circle. Unlike Focus, a point in a scanner's brackets, nothing here
 * is a bracket. */
export const MagnifyToolIcon = createLucideIcon({
  name: "magnify",
  node: [
    [
      "path",
      {
        d: "M6 13H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2v2",
        key: "area",
      },
    ],
    ["circle", { cx: "16", cy: "16", key: "lens", r: "6" }],
    ["path", { d: "M10.084 17H15a2 2 0 0 0 2-2v-4.916", key: "enlarged" }],
  ],
  size: 24,
});
export const ImageToolIcon = Image;
