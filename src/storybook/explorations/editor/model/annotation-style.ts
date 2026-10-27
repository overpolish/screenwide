// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type AnnotationHead } from "../../../../bindings/AnnotationHead";
import {
  DEFAULT_ANNOTATION_COUNTER_SIZE,
  DEFAULT_ANNOTATION_WIDTH,
} from "../../../../features/annotations/widths";
import { type Element } from "../../ideas/ideas-data";
import { corner } from "../../ideas/look-pens-model";
import { type Pen } from "../../ideas/looks";

// How an annotation is drawn. There is one house style; what changes is an
// annotation's own colour and sizes, the same choices the live overlay's
// pens offer. Each pen has a style of its own that new annotations take,
// and changing a placed annotation changes that one alone.

export type AnnotationStyle = {
  /** `#rrggbb`. A focus has none: it takes the rest away. */
  color: string;
  /** Degrees clockwise from twelve o'clock that a counter's tail points;
   * null aims it at what it counts. */
  counterAngle: number | null;
  /** A counter's disc, in points. */
  counterSize: number;
  /** Whether a focus dims what lies round it, rather than blurring it. */
  focusDims: boolean;
  /** A focus's corners, as a percentage of its box's shorter side. */
  focusRadius: number;
  head: AnnotationHead;
  /** An arrow's stroke, in points. */
  width: number;
};

export const DEFAULT_STYLE: AnnotationStyle = {
  color: "#ff383c",
  counterAngle: null,
  counterSize: DEFAULT_ANNOTATION_COUNTER_SIZE,
  focusDims: true,
  focusRadius: 25,
  head: "end",
  width: DEFAULT_ANNOTATION_WIDTH,
};

/** What each pen's style is made of, so making every annotation of a kind
 * match one copies only what that kind draws with. */
export const STYLE_OF: Record<Pen, (keyof AnnotationStyle)[]> = {
  arrow: ["color", "head", "width"],
  counter: ["color", "counterAngle", "counterSize"],
  focus: ["focusDims", "focusRadius"],
};

/** A style's part that a pen draws with. */
export const penStyle = (pen: Pen, style: AnnotationStyle) =>
  Object.fromEntries(
    STYLE_OF[pen].map((key) => [key, style[key]]),
  ) as Partial<AnnotationStyle>;

/** The pens' names, one and many, as the island says them. */
export const PEN_NAMES: Record<Pen, { many: string; one: string }> = {
  arrow: { many: "Arrows", one: "Arrow" },
  counter: { many: "Counters", one: "Counter" },
  focus: { many: "Focuses", one: "Focus" },
};

/** The way a counter's tail points unless set, in degrees clockwise from
 * twelve o'clock: from the corner it sits on into what it counts. */
export const aimedAngle = (element: Element) => {
  const { lean } = corner(element);
  return (Math.atan2(-lean.x, lean.y) * 180) / Math.PI;
};
