// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSyncExternalStore } from "react";

import {
  DEFAULT_BLUR_STRENGTH,
  defaultAnnotationSize,
  SOFT_SPOTLIGHT_EDGE,
} from "../../components/shared/annotation-style/widths";

import { AnnotationStyle } from "./annotations";

import type { AnnotationKind } from "../../components/shared/annotation-style/types";

export type AnnotationTool = AnnotationKind | "marquee" | "select";

/**
 * The dress the next annotation is drawn in: whatever the last one was changed
 * to.
 *
 * Choosing a colour once and drawing five annotations in it is the whole point
 * of the control, so the style is remembered rather than re-chosen. It lives in
 * the editor window for as long as that window does, and rides along with the
 * preview's layout so the native tool draws a fresh annotation in it without a
 * round trip of its own. The very first annotation has no remembered style:
 * Rust dresses it in the tool's own first colour at the tool's own size.
 *
 * The colour is shared between the shapes - a counter dropped after a red arrow
 * is red - but the size is not: an arrow's stroke, a counter's disc and a text
 * box's type are different measurements of different things, and eight points
 * of stroke would be a disc too small to hold a number. The alignment, which
 * only a text box reads, rides with the colour.
 *
 * A redaction keeps a dress of its own. Its colour fills a box rather than
 * marking something out, so a box filled in the last arrow's colour, or an
 * arrow drawn in the black a box was filled with, would both be surprises. A
 * highlight keeps its own too: a highlighter is its own pen, which stays
 * yellow when the arrows turn red, and is the only kind drawn by hand. So
 * does a spotlight, which is no colour at all, and whose corners and fade
 * are its own rather than a shape's. A magnifier keeps its own as well: its
 * rim frames a picture rather than marking it out, and its zoom, cone and
 * shadow mean nothing to any other kind.
 */
type DressGroup = "highlight" | "magnify" | "redact" | "shared" | "spotlight";
const lastUsed = new Map<DressGroup, AnnotationStyle>();
const dressGroup = (kind: AnnotationKind): DressGroup =>
  kind === "redact" ||
  kind === "highlight" ||
  kind === "spotlight" ||
  kind === "magnify"
    ? kind
    : "shared";
const lastSize = new Map<AnnotationKind, number>();
/**
 * And whether it animated. This is the annotation's own property rather than
 * part of its dress, so it is remembered beside the style rather than inside
 * it, and it means nothing to a screenshot, which has no clip to animate over.
 */
let lastAnimated: boolean | null = null;
/**
 * And where the last counter's tail pointed, so a second counter is dropped
 * aiming the way the first one was turned to. Also the annotation's own
 * property rather than part of its dress.
 */
let lastAngle: number | null = null;
const listeners = new Set<() => void>();

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

const sameDress = (
  held: AnnotationStyle,
  style: AnnotationStyle,
  width: number,
) =>
  held.align === style.align &&
  held.blur === style.blur &&
  held.color === style.color &&
  held.handDrawn === style.handDrawn &&
  held.head === style.head &&
  held.manual === style.manual &&
  held.radius === style.radius &&
  held.redaction === style.redaction &&
  held.shadow === style.shadow &&
  held.softness === style.softness &&
  held.strength === style.strength &&
  held.width === width;

/** Remember what the last edit to an annotation of `kind` settled on. */
export const rememberAnnotationStyle = (
  style: AnnotationStyle,
  kind: AnnotationKind = "arrow",
) => {
  const group = dressGroup(kind);
  const held = lastUsed.get(group);
  if (
    held !== undefined &&
    sameDress(held, style, held.width) &&
    lastSize.get(kind) === style.width
  )
    return;
  lastUsed.set(group, { ...style });
  lastSize.set(kind, style.width);
  for (const listener of listeners) listener();
};

/** Remember where the last counter's tail was turned to, in radians. */
export const rememberAnnotationAngle = (angle: number) => {
  if (lastAngle === angle || !Number.isFinite(angle)) return;
  lastAngle = angle;
  for (const listener of listeners) listener();
};

/** Remember whether the last annotation edit animated. */
export const rememberAnnotationAnimated = (animated: boolean) => {
  if (lastAnimated === animated) return;
  lastAnimated = animated;
  for (const listener of listeners) listener();
};

/**
 * The style a fresh annotation of `kind` is drawn in, or null while nothing has
 * been settled on and the tool's own first dress stands.
 *
 * A colour settled on for one shape dresses the others, at that shape's own
 * remembered size - or at its default, where it has none yet. A redaction is
 * dressed only by another redaction, a highlight by another highlight, a
 * spotlight by another spotlight and a magnifier by another magnifier.
 */
export const useAnnotationDefaults = (kind: AnnotationKind = "arrow") =>
  useSyncExternalStore(subscribe, () => {
    const held = lastUsed.get(dressGroup(kind));
    return held === undefined ? null : styleFor(kind, held, lastSize.get(kind));
  });

/** A fresh spotlight's corner radius, as a percentage of its shorter side.
 * The twin of `NEW_SPOTLIGHT_RADIUS` in
 * `src-tauri/src/editor/annotations/spotlight`. */
const SPOTLIGHT_RADIUS = 12;

/**
 * The dress a tool draws in before anything has been settled on: the palette's
 * yellow, or black for a redaction's fill and a spotlight's shade, at the
 * tool's own first size, with an arrow's head at its end. A magnifier starts
 * round, white-rimmed and shadowed.
 * The twins of `default_arrow_style`, `default_counter_style`,
 * `default_text_style`, `default_redact_style`, `default_highlight_style`,
 * `default_shape_style`, `default_spotlight_style` and
 * `default_magnify_style` in `src-tauri/src/editor/annotations`, which dress
 * a fresh annotation where the editor sends no dress of its own.
 */
export const firstAnnotationDress = (
  kind: AnnotationKind,
): AnnotationStyle => ({
  align: "left",
  blur: false,
  color:
    kind === "redact" || kind === "spotlight"
      ? "#000000"
      : kind === "magnify"
        ? "#ffffff"
        : "#ffcc00",
  handDrawn: false,
  head: kind === "arrow" ? "end" : "none",
  manual: false,
  radius: kind === "spotlight" ? SPOTLIGHT_RADIUS : kind === "magnify" ? 50 : 0,
  redaction: "erase",
  shadow: kind === "magnify",
  softness: kind === "spotlight" ? SOFT_SPOTLIGHT_EDGE : 0,
  strength: kind === "redact" ? DEFAULT_BLUR_STRENGTH : 0,
  width: defaultAnnotationSize(kind),
});

/** Held outside the snapshot so an unchanged store keeps returning the same
 * object: `useSyncExternalStore` compares by identity. */
const dressed = new Map<AnnotationKind, AnnotationStyle>();

const styleFor = (
  kind: AnnotationKind,
  style: AnnotationStyle,
  size: number | undefined,
) => {
  const width = size ?? defaultAnnotationSize(kind);
  const held = dressed.get(kind);
  if (held && sameDress(held, style, width)) return held;
  const next = { ...style, width };
  dressed.set(kind, next);
  return next;
};

/** Whether a fresh annotation animates, or null while none has been settled on
 * and the tool's own default - animating - stands. */
export const useAnnotationAnimatedDefault = () =>
  useSyncExternalStore(subscribe, () => lastAnimated);

/** Where a fresh counter's tail points, or null while none has been turned
 * and the tool's own default - east - stands. */
export const useAnnotationAngleDefault = () =>
  useSyncExternalStore(subscribe, () => lastAngle);
