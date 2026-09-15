// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingAnnotationClip } from "./recording-annotations";
import {
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineSourceToOutput,
} from "./recording-timeline-edit";

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
  return {
    ...clip,
    endMs,
    startMs,
  };
};

export const resizeRecordingAnnotationClip = ({
  clips,
  edge,
  edit,
  id,
  output,
  sourceDurationMs,
}: {
  clips: RecordingAnnotationClip[];
  edge: "startMs" | "endMs";
  edit: RecordingTimelineEdit;
  id: string;
  output: number;
  sourceDurationMs: number;
}) => {
  const source = Math.round(
    recordingTimelineOutputToSource(edit, Math.max(0, Math.min(1, output))) *
      sourceDurationMs,
  );
  return clips.map((clip) =>
    clip.annotation.id !== id
      ? clip
      : {
          ...clip,
          [edge]:
            edge === "startMs"
              ? Math.max(0, Math.min(source, clip.endMs - 1))
              : Math.min(sourceDurationMs, Math.max(source, clip.startMs + 1)),
        },
  );
};
