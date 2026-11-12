// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback } from "react";

import { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import {
  recordingTimelinePlaybackRangeFrom,
  recordingTimelinePlaybackRanges,
  recordingTimelinePlaybackRangesFrom,
  type RecordingTimelinePlaybackRange,
} from "../timeline/recording-timeline-playback";

import type { PreviewSoundTrack } from "./recording-preview-playback-api";

const playedMs = (ranges: RecordingTimelinePlaybackRange[]) =>
  ranges.reduce(
    (total, range) =>
      total + (range.sourceEndMs - range.sourceStartMs) / range.playbackRate,
    0,
  );

/** The stretches of the recording the preview plays, as edited, the one a
 * source position falls in, and what a play from a source position sends:
 * the stretches from there on, and the songs and sound effects with where
 * on the timeline playback starts. */
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
  const playFrom = useCallback(
    (sourcePositionMs: number) => {
      const all = playbackRanges();
      const ranges = recordingTimelinePlaybackRangesFrom(all, sourcePositionMs);
      const edit = timelineEditRef.current;
      const clips = edit?.audioClips ?? [];
      const timelineMs = Math.round(playedMs(all));
      const soundTrack: PreviewSoundTrack | undefined =
        clips.length > 0
          ? {
              clips,
              muted: edit?.mutedAudio,
              startMs: Math.max(0, timelineMs - Math.round(playedMs(ranges))),
              timelineMs,
            }
          : undefined;
      return { playbackRanges: ranges, soundTrack };
    },
    [playbackRanges, timelineEditRef],
  );
  return { playFrom, playbackRangeFrom, playbackRanges };
}
