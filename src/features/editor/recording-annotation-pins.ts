// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { annotationDrawInMs } from "./annotation-kinds";
import {
  RecordingAnnotationClip,
  RecordingAnnotationPin,
} from "./recording-annotations";

/** How close to a keyframe, in source milliseconds, a moment is still that
 * keyframe's. The twin of `KEYFRAME_SLACK_MS` in `pin/model.rs`. */
const KEYFRAME_SLACK_MS = 8;

/** Whether `clip` can follow its content. Only the screen's annotations can:
 * the tracker follows the screen recording. */
export const isPinnable = (clip: RecordingAnnotationClip) =>
  clip.trackId === "primary";

/**
 * `clip` pinned at `positionMs`: the frame under the playhead when it falls in
 * the clip, and otherwise the moment the annotation was placed, once it has
 * drawn in.
 */
export const pinnedClip = (
  clip: RecordingAnnotationClip,
  positionMs: number,
): RecordingAnnotationClip => {
  const placed = Math.min(
    clip.endMs - 1,
    clip.startMs + annotationDrawInMs(clip.annotation),
  );
  const inside = positionMs >= clip.startMs && positionMs < clip.endMs;
  const pinnedMs = Math.max(0, Math.round(inside ? positionMs : placed));
  return {
    ...clip,
    pin: { keyframes: [{ dx: 0, dy: 0, ms: pinnedMs }], pinnedMs },
  };
};

/** `clip` with its pin taken away: it stays where it was drawn. */
export const unpinnedClip = ({
  pin: _pin,
  ...clip
}: RecordingAnnotationClip): RecordingAnnotationClip => clip;

/** How many places `pin` was put by hand besides the one it was made on. */
export const pinCorrections = (pin: RecordingAnnotationPin) =>
  pin.keyframes.filter(
    (keyframe) => Math.abs(keyframe.ms - pin.pinnedMs) > KEYFRAME_SLACK_MS,
  ).length;

/** `pin` with every correction taken away, keeping the keyframe it was made
 * on. */
export const clearedPinCorrections = (
  pin: RecordingAnnotationPin,
): RecordingAnnotationPin => ({
  keyframes: pin.keyframes.filter(
    (keyframe) => Math.abs(keyframe.ms - pin.pinnedMs) <= KEYFRAME_SLACK_MS,
  ),
  pinnedMs: pin.pinnedMs,
});

/** `pin` moved `deltaMs` later, as its clip is. */
export const shiftedPin = (
  pin: RecordingAnnotationPin,
  deltaMs: number,
): RecordingAnnotationPin => ({
  keyframes: pin.keyframes.map((keyframe) => ({
    ...keyframe,
    ms: Math.max(0, Math.round(keyframe.ms + deltaMs)),
  })),
  pinnedMs: Math.max(0, Math.round(pin.pinnedMs + deltaMs)),
});

/** `pin` without its keyframe at `ms`. The last keyframe is the pin itself
 * and is kept; taking the one it was made on hands that role to the nearest
 * one left. */
export const withoutPinKeyframe = (
  pin: RecordingAnnotationPin,
  ms: number,
): RecordingAnnotationPin => {
  const keyframes = pin.keyframes.filter((keyframe) => keyframe.ms !== ms);
  if (keyframes.length === 0) return pin;
  const nearest = keyframes.reduce((best, keyframe) =>
    Math.abs(keyframe.ms - ms) < Math.abs(best.ms - ms) ? keyframe : best,
  );
  return {
    keyframes,
    pinnedMs: ms === pin.pinnedMs ? nearest.ms : pin.pinnedMs,
  };
};
