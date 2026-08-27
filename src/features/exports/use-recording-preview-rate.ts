// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Dispatch,
  RefObject,
  SetStateAction,
  useCallback,
  useRef,
  useState,
} from "react";

import { playRecordingPreview } from "./recording-preview-playback-api";
import { RecordingTimelineEdit } from "./recording-timeline-edit";
import {
  recordingTimelinePlaybackRangeFrom,
  recordingTimelinePlaybackRanges,
  recordingTimelinePlaybackRangesFrom,
} from "./recording-timeline-playback";

export function useRecordingPreviewRate({
  isEnabled,
  isPlayingRef,
  positionRef,
  sessionIdRef,
  setError,
  setIsPlaying,
  timelineEditRef,
  timingRef,
  wantsPlaybackRef,
}: {
  isEnabled: boolean;
  isPlayingRef: RefObject<boolean>;
  positionRef: RefObject<number>;
  sessionIdRef: RefObject<number>;
  setError: Dispatch<SetStateAction<string | null>>;
  setIsPlaying: Dispatch<SetStateAction<boolean>>;
  timelineEditRef: RefObject<RecordingTimelineEdit | null | undefined>;
  timingRef: RefObject<[durationMs: number, framesPerSecond: number | null]>;
  wantsPlaybackRef: RefObject<boolean>;
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
  const playbackRateRef = useRef(1);
  const [playbackRate, setPlaybackRate] = useState(1);
  const changePlaybackRate = useCallback(
    (nextRate: number) => {
      if (
        !Number.isFinite(nextRate) ||
        nextRate < 0.25 ||
        nextRate > 4 ||
        nextRate === playbackRateRef.current
      )
        return;
      playbackRateRef.current = nextRate;
      setPlaybackRate(nextRate);
      if (!isEnabled || !isPlayingRef.current) return;
      const { index, ranges } = playbackRangeFrom(positionRef.current);
      void playRecordingPreview(
        sessionIdRef.current,
        Math.round(ranges[index].sourceEndMs),
        {
          playbackRanges: recordingTimelinePlaybackRangesFrom(
            playbackRanges(),
            positionRef.current,
          ),
          playbackRate: nextRate,
          startPositionMs: Math.round(positionRef.current),
        },
      ).catch((cause: unknown) => {
        wantsPlaybackRef.current = false;
        isPlayingRef.current = false;
        setIsPlaying(false);
        setError(String(cause));
      });
    },
    [
      isEnabled,
      isPlayingRef,
      playbackRangeFrom,
      playbackRanges,
      positionRef,
      sessionIdRef,
      setError,
      setIsPlaying,
      wantsPlaybackRef,
    ],
  );
  return {
    playbackRangeFrom,
    playbackRanges,
    playbackRate,
    playbackRateRef,
    setPlaybackRate: changePlaybackRate,
  };
}
