// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { RefObject, useCallback } from "react";

import {
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
} from "./editing/recording-timeline-edit";
import { recordingTimelineOutputToSourceStart } from "./editing/recording-timeline-source-order";
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
      // The frame that starts playing there: at a join, the stretch after
      // it, which a gap's end otherwise left for the gap's own empty end.
      player.current(
        recordingTimelineOutputToSourceStart(edit.segments, ratio) * durationMs,
        phase,
        annotationClips,
      );
    },
    [edit, playhead, player, durationMs],
  );
}
