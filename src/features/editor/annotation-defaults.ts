// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSyncExternalStore } from "react";

import { defaultAnnotationSize } from "../../components/shared/annotation-style/widths";

import { AnnotationStyle } from "./annotations";

/** Which shape a tool draws, which is what a remembered size belongs to. */
export type AnnotationKind = "arrow" | "counter";

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
 * is red - but the size is not: an arrow's stroke and a counter's disc are
 * different measurements of different things, and eight pixels of stroke would
 * be a disc too small to hold a number.
 */
let lastUsed: AnnotationStyle | null = null;
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

/** Remember what the last edit to an annotation of `kind` settled on. */
export const rememberAnnotationStyle = (
  style: AnnotationStyle,
  kind: AnnotationKind = "arrow",
) => {
  if (
    lastUsed?.color === style.color &&
    lastUsed.head === style.head &&
    lastSize.get(kind) === style.width
  )
    return;
  lastUsed = { color: style.color, head: style.head, width: style.width };
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
 * A colour settled on for one shape dresses the other, at that shape's own
 * remembered size - or at its default, where it has none yet.
 */
export const useAnnotationDefaults = (kind: AnnotationKind = "arrow") =>
  useSyncExternalStore(subscribe, () =>
    lastUsed === null ? null : styleFor(kind, lastUsed, lastSize.get(kind)),
  );

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
  if (
    held &&
    held.color === style.color &&
    held.head === style.head &&
    held.width === width
  )
    return held;
  const next = { color: style.color, head: style.head, width };
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
