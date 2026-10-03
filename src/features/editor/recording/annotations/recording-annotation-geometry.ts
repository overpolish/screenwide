// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineSourceToOutput,
} from "../../timeline/editing/recording-timeline-edit";

import { shiftedPin } from "./recording-annotation-pins";
import { RecordingAnnotationClip } from "./recording-annotations";

/** Moves a clip by output time while keeping its visible output duration. */
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
  const duration = Math.max(0, sourceDurationMs);
  if (duration <= 0) return clip;
  const start = recordingTimelineSourceToOutput(edit, clip.startMs / duration);
  const end = recordingTimelineSourceToOutput(edit, clip.endMs / duration);
  const outputDuration = end - start;
  const shiftedStart = Math.max(
    0,
    Math.min(1 - outputDuration, start + deltaOutput),
  );
  const shiftedEnd = shiftedStart + outputDuration;
  let startMs = Math.round(
    recordingTimelineOutputToSource(edit, shiftedStart) * duration,
  );
  let endMs = Math.round(
    recordingTimelineOutputToSource(edit, shiftedEnd) * duration,
  );
  if (endMs <= startMs) {
    endMs = Math.min(duration, startMs + 1);
    startMs = Math.max(0, endMs - 1);
  }
  // A pinned clip's keyframes move with it: it follows whatever sits where
  // it now is, at the same point in its clip.
  return {
    ...clip,
    endMs,
    pin: clip.pin && shiftedPin(clip.pin, startMs - clip.startMs),
    startMs,
  };
};

/**
 * `clips` with the `edge` of every clip named in `ids` trimmed by the output
 * time that takes the pressed clip `id`'s edge to `output`, so a choice of
 * several trims together. The shift stops for all of them where the first
 * would shrink below a millisecond or pass an end of the video.
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
  if (!pressed || sourceDurationMs <= 0) return clips;
  const outputOf = (ms: number) =>
    recordingTimelineSourceToOutput(edit, ms / sourceDurationMs);
  let least = -Infinity;
  let most = Infinity;
  for (const clip of clips) {
    if (!ids.has(clip.annotation.id)) continue;
    const at = outputOf(clip[edge]);
    const [low, high] =
      edge === "startMs"
        ? [0, outputOf(clip.endMs - 1)]
        : [outputOf(clip.startMs + 1), 1];
    least = Math.max(least, low - at);
    most = Math.min(most, high - at);
  }
  const shift = Math.max(
    least,
    Math.min(most, Math.max(0, Math.min(1, output)) - outputOf(pressed[edge])),
  );
  return clips.map((clip) => {
    if (!ids.has(clip.annotation.id)) return clip;
    const source = Math.round(
      recordingTimelineOutputToSource(
        edit,
        Math.max(0, Math.min(1, outputOf(clip[edge]) + shift)),
      ) * sourceDurationMs,
    );
    return {
      ...clip,
      [edge]:
        edge === "startMs"
          ? Math.max(0, Math.min(source, clip.endMs - 1))
          : Math.min(sourceDurationMs, Math.max(source, clip.startMs + 1)),
    };
  });
};
