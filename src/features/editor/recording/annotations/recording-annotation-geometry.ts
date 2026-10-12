// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import {
  recordingMsAt,
  recordingTimelineLengthMs,
} from "../../timeline/editing/recording-timeline-time";

import { heldLengthMs } from "./image-once";
import { shiftedPin } from "./recording-annotation-pins";
import { RecordingAnnotationClip } from "./recording-annotations";

/** Moves a clip by `deltaOutput`, a share of the timeline, keeping its
 * length and staying on the timeline. */
export const moveRecordingAnnotationClip = ({
  clip,
  deltaOutput,
  edit,
  sourceDurationMs,
}: {
  clip: RecordingAnnotationClip;
  deltaOutput: number;
  edit: RecordingTimelineEdit;
  sourceDurationMs: number;
}): RecordingAnnotationClip => {
  const lengthMs = recordingTimelineLengthMs(edit, sourceDurationMs);
  if (lengthMs <= 0) return clip;
  const span = clip.endMs - clip.startMs;
  const startMs = Math.round(
    Math.max(
      0,
      Math.min(lengthMs - span, clip.startMs + deltaOutput * lengthMs),
    ),
  );
  if (startMs === clip.startMs) return clip;
  // A pinned clip's keyframes move with it: it follows whatever sits where
  // it now is, at the same point in its clip. They are in recording time.
  const recordingShift =
    recordingMsAt(edit, sourceDurationMs, startMs) -
    recordingMsAt(edit, sourceDurationMs, clip.startMs);
  return {
    ...clip,
    endMs: startMs + span,
    pin: clip.pin && shiftedPin(clip.pin, recordingShift),
    startMs,
  };
};

/**
 * `clips` with the `edge` of every clip named in `ids` trimmed by the time
 * that takes the pressed clip `id`'s edge to `output`, a share of the
 * timeline, so a choice of several trims together. The shift stops for all
 * of them where the first would shrink below a millisecond or leave the
 * timeline. A clip held to a length of its own is left as it is.
 */
export const trimRecordingAnnotationClips = ({
  clips,
  edge,
  edit,
  id,
  ids,
  output,
  sourceDurationMs,
}: {
  clips: RecordingAnnotationClip[];
  edge: "startMs" | "endMs";
  edit: RecordingTimelineEdit;
  id: string;
  ids: ReadonlySet<string>;
  output: number;
  sourceDurationMs: number;
}) => {
  const pressed = clips.find((clip) => clip.annotation.id === id);
  const lengthMs = recordingTimelineLengthMs(edit, sourceDurationMs);
  if (!pressed || lengthMs <= 0) return clips;
  let least = -Infinity;
  let most = Infinity;
  for (const clip of clips) {
    if (!ids.has(clip.annotation.id) || heldLengthMs(clip) !== null) continue;
    const [low, high] =
      edge === "startMs" ? [0, clip.endMs - 1] : [clip.startMs + 1, lengthMs];
    least = Math.max(least, low - clip[edge]);
    most = Math.min(most, high - clip[edge]);
  }
  const shift = Math.round(
    Math.max(
      least,
      Math.min(
        most,
        Math.max(0, Math.min(1, output)) * lengthMs - pressed[edge],
      ),
    ),
  );
  return clips.map((clip) =>
    !ids.has(clip.annotation.id) || heldLengthMs(clip) !== null
      ? clip
      : { ...clip, [edge]: clip[edge] + shift },
  );
};
