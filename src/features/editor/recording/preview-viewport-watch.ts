// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** How long after the preview's viewport last moved or resized its layout
 * is still handed on mid-gesture, in ms: long enough for the marker to be
 * placed again once the viewport has settled. */
const SETTLE_MS = 250;

/**
 * Whether the preview's layout is handed to the native surface now. Not
 * while a pointer-rate gesture is under way, which the native workspace
 * draws itself (feeding React's mirrored markers back makes the two fight),
 * unless the viewport itself moved or resized meanwhile: the timeline below
 * growing, say, as a version line comes in. Then native has to follow, or
 * the picture would hang over the timeline until the gesture ends.
 */
export function createViewportWatch(
  isGesturing: () => boolean,
  marker: () => Element | null,
) {
  let last: string | null = null;
  let movedAt = -Infinity;
  return () => {
    const viewport = marker()?.closest("[data-recording-preview-viewport]");
    const rect = viewport?.getBoundingClientRect();
    const seen = rect
      ? `${String(rect.left)},${String(rect.top)},${String(rect.width)},${String(rect.height)}`
      : null;
    if (seen !== last) {
      last = seen;
      movedAt = performance.now();
    }
    return !isGesturing() || performance.now() - movedAt < SETTLE_MS;
  };
}
