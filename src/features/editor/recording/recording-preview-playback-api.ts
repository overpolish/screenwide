// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { AudioClipKind } from "../../../bindings/AudioClipKind";
import type { RecordingAudioClip } from "../../../bindings/RecordingAudioClip";
import type { RecordingTimelinePlaybackRange } from "../timeline/recording-timeline-playback";

/** The songs and sound effects a play hears, the kinds left out, and where
 * on the timeline it starts, in ms of the edited video. */
export type PreviewSoundTrack = {
  clips: RecordingAudioClip[];
  startMs: number;
  timelineMs: number;
  muted?: AudioClipKind[];
};

export const playRecordingPreview = (
  sessionId: number,
  playbackEndMs?: number,
  options?: {
    playbackRanges?: RecordingTimelinePlaybackRange[];
    soundTrack?: PreviewSoundTrack;
    startPositionMs?: number;
  },
) => {
  const { playbackRanges, soundTrack, startPositionMs } = options ?? {};
  return invoke<null>("play_recording_preview", {
    playbackEndMs,
    playbackRanges,
    sessionId,
    soundTrack,
    startPositionMs,
  });
};
