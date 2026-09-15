// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { RefObject, useCallback } from "react";

import {
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
} from "../recording-timeline-edit";

import { Playhead } from "./scrub-playhead";
import { SeekHandler } from "./timeline-seek";

export function useTimelineSeek({
  durationMs,
  edit,
  player,
  playhead,
}: {
  durationMs: number;
  edit: RecordingTimelineEdit;
  player: RefObject<SeekHandler>;
  playhead: Playhead;
}) {
  return useCallback<SeekHandler>(
    (ratio, phase, annotationClips) => {
      playhead.publish(
        (ratio * durationMs * recordingTimelineRetainedDuration(edit)) / 1000,
        ratio,
      );
      player.current(
        recordingTimelineOutputToSource(edit, ratio) * durationMs,
        phase,
        annotationClips,
      );
    },
    [edit, playhead, player, durationMs],
  );
}
