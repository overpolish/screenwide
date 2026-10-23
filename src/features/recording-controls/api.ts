// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel, invoke } from "@tauri-apps/api/core";

import {
  RecordingSnapshot,
  ReplaySnapshot,
  StartRecordingOptions,
} from "./types";

export const getRecordingSnapshot = () =>
  invoke<RecordingSnapshot>("get_recording_snapshot");

/** How much of the running recording is on disk, or null outside one. */
export const getRecordingWrittenBytes = () =>
  invoke<number | null>("get_recording_written_bytes");

// Every lifecycle call is awaited: Rust rejects illegal transitions, and a
// swallowed rejection would leave the UI claiming a state that does not exist.
export const startRecording = async (options: StartRecordingOptions) => {
  await invoke<null>("start_recording", { options });
};

export const pauseRecording = async () => {
  await invoke<null>("pause_recording");
};

export const resumeRecording = async () => {
  await invoke<null>("resume_recording");
};

export const stopRecording = async () => {
  await invoke<null>("stop_recording");
};

export const cancelRecording = async () => {
  await invoke<null>("cancel_recording");
};

export const startRecordingMonitor = (
  subscriptionId: number,
  channel: Channel<ArrayBuffer>,
) => invoke<null>("start_recording_monitor", { channel, subscriptionId });

export const stopRecordingMonitor = (subscriptionId: number) =>
  invoke<null>("stop_recording_monitor", { subscriptionId });

export const getReplaySnapshot = () =>
  invoke<ReplaySnapshot>("get_replay_snapshot");

/** Starts the replay buffer with the recording controls' settings as they
 * are now. */
export const startReplayBuffer = async (options: StartRecordingOptions) => {
  await invoke<null>("start_replay_buffer", { options });
};

export const stopReplayBuffer = async () => {
  await invoke<null>("stop_replay_buffer");
};
