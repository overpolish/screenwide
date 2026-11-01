// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback } from "react";

import { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import {
  recordingTimelinePlaybackRangeFrom,
  recordingTimelinePlaybackRanges,
} from "../timeline/recording-timeline-playback";

/** The stretches of the recording the preview plays, as edited, and the one
 * a source position falls in. */
export function useRecordingPreviewRanges({
  timelineEditRef,
  timingRef,
}: {
  timelineEditRef: RefObject<RecordingTimelineEdit | null | undefined>;
  timingRef: RefObject<[durationMs: number, framesPerSecond: number | null]>;
}) {
  const playbackRanges = useCallback(
    () =>
      recordingTimelinePlaybackRanges(
        timelineEditRef.current,
        timingRef.current[0],
      ),
    [timelineEditRef, timingRef],
  );
  const playbackRangeFrom = useCallback(
    (sourcePositionMs: number) =>
      recordingTimelinePlaybackRangeFrom(playbackRanges(), sourcePositionMs),
    [playbackRanges],
  );
  return { playbackRangeFrom, playbackRanges };
}
