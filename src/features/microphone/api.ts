// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

/** Whether the microphone of a recording or of notes is muted. */
export const getMicrophoneMuted = () => invoke<boolean>("microphone_muted");

export const setMicrophoneMuted = (muted: boolean) =>
  invoke<null>("set_microphone_muted", { muted });

/** Sent to every window with whether the microphone is muted. */
export const MICROPHONE_MUTED_EVENT = "microphone://muted";
