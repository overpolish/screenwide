// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import {
  getMicrophoneMuted,
  MICROPHONE_MUTED_EVENT,
  setMicrophoneMuted,
} from "./api";

/** The microphone of a recording or of notes as the island shows it:
 * whether it is muted, by its button, its shortcut or another window, and a
 * way to mute it. Read once as the page starts listening, so a mute told
 * before is not missed. */
export function useMicrophoneMute() {
  const [isMuted, setIsMuted] = useState(false);
  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<boolean>(MICROPHONE_MUTED_EVENT, ({ payload }) => {
      setIsMuted(payload);
    }).then((handle) => {
      if (disposed) handle();
      else unlisten = handle;
    });
    getMicrophoneMuted()
      .then((muted) => {
        if (!disposed) setIsMuted(muted);
      })
      .catch((cause: unknown) => {
        console.error("Could not read whether the microphone is muted", cause);
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  return {
    isMuted,
    onMutedChange: (muted: boolean) => {
      setMicrophoneMuted(muted).catch((cause: unknown) => {
        console.error("Could not mute the microphone", cause);
      });
    },
  };
}
