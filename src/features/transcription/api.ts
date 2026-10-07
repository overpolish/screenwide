// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { TranscriptionState } from "./types";

export const getTranscriptionState = () =>
  invoke<TranscriptionState>("get_transcription_state");

export const downloadTranscriptionModel = (model: string) =>
  invoke<null>("download_transcription_model", { model });

export const cancelTranscriptionDownload = (model: string) =>
  invoke<null>("cancel_transcription_download", { model });

export const removeTranscriptionModel = (model: string) =>
  invoke<null>("remove_transcription_model", { model });

export const setTranscriptionLanguage = (language: string) =>
  invoke<null>("set_transcription_language", { language });

/** Every change to the models or the language, download progress included,
 * so each window shows one state. */
const TRANSCRIPTION_CHANGED_EVENT = "transcription://changed";

export const listenToTranscription = (
  onChange: (state: TranscriptionState) => void,
) =>
  listen<TranscriptionState>(TRANSCRIPTION_CHANGED_EVENT, (event) => {
    onChange(event.payload);
  });
