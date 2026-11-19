// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect, useRef } from "react";

import { useRecordingStore } from "../recording-controls/store";

import type { RecordingStatus } from "../../bindings/RecordingStatus";

/** Whether the teleprompter plays as a recording moves from `from` to
 * `to`: it starts as the countdown ends, pauses and resumes with the
 * recording, and pauses as it stops. Null leaves it as it is. */
export const playsAcross = (
  from: RecordingStatus,
  to: RecordingStatus,
): boolean | null => {
  if (from === to) return null;
  if (to === "recording") return from === "starting" || from === "paused";
  if (to === "paused" || to === "stopping" || to === "idle")
    return from === "recording" ? false : null;
  return null;
};

/**
 * Plays and pauses the teleprompter with the recording, from wherever its
 * place stands, so a take starts on the line set up for it. A recording
 * already under way when the teleprompter opens moves nothing until it
 * changes.
 */
export function useRecordingLink(
  onPlayingChange: (isPlaying: boolean) => void,
) {
  const onChangeRef = useRef(onPlayingChange);
  onChangeRef.current = onPlayingChange;
  useEffect(
    () =>
      useRecordingStore.subscribe((state, previous) => {
        if (!previous.hydrated) return;
        const plays = playsAcross(
          previous.snapshot.status,
          state.snapshot.status,
        );
        if (plays !== null) onChangeRef.current(plays);
      }),
    [],
  );
}
