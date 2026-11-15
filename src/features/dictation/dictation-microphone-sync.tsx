// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect } from "react";

import { useRecordingInputStore } from "../recording-inputs/store";

import { setDictationMicrophone } from "./api";

/** Tells dictation which microphone the recording island has chosen, as the
 * island's page loads and whenever the choice changes, in this window or
 * another. */
export function DictationMicrophoneSync() {
  const microphone = useRecordingInputStore(
    (state) => state.selectedMicrophone?.id ?? null,
  );
  useEffect(() => {
    if (!isTauri()) return;
    setDictationMicrophone(microphone).catch((cause: unknown) => {
      console.error("Could not tell dictation the microphone", cause);
    });
  }, [microphone]);
  return null;
}
