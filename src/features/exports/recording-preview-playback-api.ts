// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { RecordingTimelinePlaybackRange } from "./recording-timeline-playback";

export const playRecordingPreview = (
  sessionId: number,
  playbackEndMs?: number,
  options?: {
    playbackRanges?: RecordingTimelinePlaybackRange[];
    playbackRate?: number;
    startPositionMs?: number;
  },
) => {
  const { playbackRanges, playbackRate, startPositionMs } = options ?? {};
  return invoke<null>("play_recording_preview", {
    playbackEndMs,
    playbackRanges,
    playbackRate,
    sessionId,
    startPositionMs,
  });
};
