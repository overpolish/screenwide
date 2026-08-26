// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

export const playRecordingPreview = (
  sessionId: number,
  playbackEndMs?: number,
  options?: {
    playbackRanges?: { sourceEndMs: number; sourceStartMs: number }[];
    startPositionMs?: number;
  },
) => {
  const { playbackRanges, startPositionMs } = options ?? {};
  return invoke<null>("play_recording_preview", {
    playbackEndMs,
    playbackRanges,
    sessionId,
    startPositionMs,
  });
};
