// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSyncExternalStore } from "react";

import { AnnotationStyle } from "./annotations";

/**
 * The dress the next arrow is drawn in: whatever the last one was changed to.
 *
 * Choosing a colour once and drawing five arrows in it is the whole point of
 * the control, so the style is remembered rather than re-chosen. It lives in
 * the editor window for as long as that window does, and rides along with the
 * preview's layout so the native tool draws a fresh arrow in it without a
 * round trip of its own. The very first arrow has no remembered style: Rust
 * dresses it in the operating system's accent at the tool's own stroke.
 */
let lastUsed: AnnotationStyle | null = null;
/**
 * And whether it animated. This is the mark's own property rather than part
 * of its dress, so it is remembered beside the style rather than inside it,
 * and it means nothing to a screenshot, which has no clip to animate over.
 */
let lastAnimated: boolean | null = null;
const listeners = new Set<() => void>();

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

const snapshot = () => lastUsed;

/** Remember what the last annotation edit settled on. */
export const rememberAnnotationStyle = (style: AnnotationStyle) => {
  if (
    lastUsed?.color === style.color &&
    lastUsed.head === style.head &&
    lastUsed.width === style.width
  )
    return;
  lastUsed = { color: style.color, head: style.head, width: style.width };
  for (const listener of listeners) listener();
};

/** Remember whether the last annotation edit animated. */
export const rememberAnnotationAnimated = (animated: boolean) => {
  if (lastAnimated === animated) return;
  lastAnimated = animated;
  for (const listener of listeners) listener();
};

/** The style a fresh arrow is drawn in, or null while none has been settled
 * on and the tool's own first dress stands. */
export const useAnnotationDefaults = () =>
  useSyncExternalStore(subscribe, snapshot);

/** Whether a fresh arrow animates, or null while none has been settled on
 * and the tool's own default - animating - stands. */
export const useAnnotationAnimatedDefault = () =>
  useSyncExternalStore(subscribe, () => lastAnimated);
