// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback, useEffect, useRef } from "react";

import {
  recordingTimelineSourceToOutput,
  RecordingTimelineEdit,
} from "../recording-timeline-edit";
import { recordingTimelinePlaybackDurationMs } from "../recording-timeline-playback";

import { Playhead } from "./scrub-playhead";

export function useRecordingTrimPreview({
  edit,
  playhead,
  totalDurationRef,
}: {
  playhead: Playhead;
  totalDurationRef: RefObject<number>;
  edit?: RecordingTimelineEdit | null;
}) {
  const stateRef = useRef({
    active: false,
    restorePositionMs: null as number | null,
    timeout: null as number | null,
  });
  const clear = useCallback(() => {
    const state = stateRef.current;
    state.active = false;
    state.restorePositionMs = null;
    if (state.timeout !== null) window.clearTimeout(state.timeout);
    state.timeout = null;
  }, []);
  useEffect(() => clear, [clear]);

  const onPosition = useCallback(
    (positionMs: number) => {
      const state = stateRef.current;
      if (state.active) {
        if (
          state.restorePositionMs === null ||
          Math.abs(positionMs - state.restorePositionMs) > 1
        )
          return;
        clear();
      }
      const total = totalDurationRef.current;
      const outputRatio =
        edit && total > 0
          ? recordingTimelineSourceToOutput(edit, positionMs / total)
          : total > 0
            ? positionMs / total
            : 0;
      const outputDurationMs = recordingTimelinePlaybackDurationMs(edit, total);
      playhead.publish((outputRatio * outputDurationMs) / 1_000, outputRatio);
    },
    [clear, edit, playhead, totalDurationRef],
  );
  const start = useCallback(() => {
    clear();
    stateRef.current.active = true;
  }, [clear]);
  const restore = useCallback(
    (positionMs: number) => {
      const state = stateRef.current;
      state.restorePositionMs = Math.round(positionMs);
      if (state.timeout !== null) window.clearTimeout(state.timeout);
      state.timeout = window.setTimeout(clear, 500);
    },
    [clear],
  );

  return { onPosition, restore, start };
}
