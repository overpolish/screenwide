// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { MicrophoneProgress } from "../../../../bindings/MicrophoneProgress";
import type { NoiseReduction } from "../../../../bindings/NoiseReduction";
import type { SilenceCut } from "../../../../bindings/SilenceCut";

export type { MicrophoneProgress, NoiseReduction };

/** How far a microphone tool has got, as the editor is told while it works;
 * `src-tauri/src/editor/speech/commands.rs`. */
export const listenToMicrophoneProgress = (
  onProgress: (progress: MicrophoneProgress) => void,
) =>
  listen<MicrophoneProgress>("editor://microphone-progress", (event) => {
    onProgress(event.payload);
  });

/** The long pauses Remove silences would cut from the recording open, from
 * `src-tauri/src/editor/speech/silences.rs`. */
export const planRecordingSilences = (artifactId: number) =>
  invoke<SilenceCut[]>("plan_recording_silences", { artifactId });

export const getRecordingNoiseReduction = (artifactId: number) =>
  invoke<NoiseReduction>("get_recording_noise_reduction", { artifactId });

/** Takes the microphone's noise out, or puts it back, answering with what
 * the project says now. Turning it on the first time takes a few seconds. */
export const setRecordingNoiseReduction = (
  artifactId: number,
  enabled: boolean,
) =>
  invoke<NoiseReduction>("set_recording_noise_reduction", {
    artifactId,
    enabled,
  });
