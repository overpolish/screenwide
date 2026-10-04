// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RecordingAnnotationClip } from "./recording-annotations";

/**
 * How long a clip is held to, in milliseconds: one run of a moving sticker
 * played once, which leaves as its run ends - with Animate on, leaving while
 * the run is still playing. Null for every clip whose length is its own.
 */
export const heldLengthMs = (clip: RecordingAnnotationClip) => {
  const shape = clip.annotation.shape;
  return shape.kind === "sticker" && shape.play?.once
    ? Math.max(1, Math.round(shape.play.cycleMs))
    : null;
};

/**
 * `clip` at the length it is held to, from where it starts; one too near the
 * end of the recording for a whole run starts earlier, as far as the
 * recording allows, so the run is seen whole.
 */
export const heldLengthClip = (
  clip: RecordingAnnotationClip,
  sourceDurationMs: number,
): RecordingAnnotationClip => {
  const length = heldLengthMs(clip);
  if (length === null || sourceDurationMs <= 0) return clip;
  const startMs = Math.max(
    0,
    Math.min(clip.startMs, sourceDurationMs - length),
  );
  const endMs = Math.min(sourceDurationMs, startMs + length);
  return startMs === clip.startMs && endMs === clip.endMs
    ? clip
    : { ...clip, endMs, startMs };
};
