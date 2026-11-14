// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span, type TimeMap } from "../timeline-scale";

import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";

// Where the person on camera moved, as beats. What is said is already a
// beat in lines of its own, and a gesture made while talking belongs to its
// line, so a movement makes a beat only where nothing is said. Each keeps a
// little of what led up to it and the moment after, but never reaches into
// a line, so a movement beside a line never cuts it short.

/** How long before a movement, and after it, its beat keeps, in seconds. */
const AROUND = [0.5, 1] as const;

/** Each movement, with a little either side, where no line is said. */
export function movementSpans(
  camera: readonly Span[],
  lines: readonly Span[],
): Span[] {
  return camera.flatMap(({ end, start }) => {
    const pieces: Span[] = [];
    let at = start - AROUND[0];
    const until = end + AROUND[1];
    for (const line of lines) {
      if (line.end <= at || line.start >= until) continue;
      if (line.start > at) pieces.push({ end: line.start, start: at });
      at = Math.max(at, line.end);
    }
    if (until > at) pieces.push({ end: until, start: at });
    return pieces;
  });
}

/** The movements the camera is seen in, out of `camera`: under a scene that
 * shows the screen alone, or neither, it is not. `map` places a recorded
 * moment on the timeline the scenes are timed on. */
export function movementsSeen(
  camera: readonly Span[],
  scenes: readonly RecordingSceneClip[] | undefined,
  map: TimeMap,
): Span[] {
  const hiding = (scenes ?? []).filter(
    ({ preset }) => preset === "blank" || preset === "screen-only",
  );
  if (hiding.length === 0) return [...camera];
  return camera.filter(({ start }) => {
    const playedMs = map.toPlayed(start) * 1000;
    return !hiding.some(
      ({ endMs, startMs }) => startMs <= playedMs && playedMs < endMs,
    );
  });
}
