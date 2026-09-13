// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

/**
 * Hands the native ribbon the enabled tracks' envelopes. An empty list clears
 * it, which is what a layout with video panes sends.
 */
export const setRecordingAudioVisualizer = ({
  artifactId,
  tracks,
}: {
  artifactId: number;
  tracks: {
    gainDecibels: number;
    streamIndex: number;
    waveform: number[];
  }[];
}) => invoke<null>("set_recording_audio_visualizer", { artifactId, tracks });
