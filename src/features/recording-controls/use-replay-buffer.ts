// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { getReplaySnapshot, startReplayBuffer, stopReplayBuffer } from "./api";
import { startRecordingOptions } from "./recording-request";
import { initialReplaySnapshot, ReplaySnapshot } from "./types";

const REPLAY_STATE_EVENT = "replay://state";
const REPLAY_ERROR_EVENT = "replay://error";

/**
 * The replay buffer as Rust reports it, and the switch the recording bar
 * turns it on and off with. Turning it on takes the bar's settings as they
 * are at that moment; later changes only set up the next recording.
 */
export function useReplayBuffer() {
  const [snapshot, setSnapshot] = useState<ReplaySnapshot>(
    initialReplaySnapshot,
  );

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];
    let disposed = false;
    const keep = (listener: UnlistenFn) => {
      if (disposed) listener();
      else unlisteners.push(listener);
    };

    void listen<ReplaySnapshot>(REPLAY_STATE_EVENT, ({ payload }) => {
      setSnapshot(payload);
    }).then(keep);
    // There is no toast surface; the switch follows the state Rust settles on.
    void listen<{ message: string }>(REPLAY_ERROR_EVENT, ({ payload }) => {
      console.error(`Replay buffer: ${payload.message}`);
    }).then(keep);
    getReplaySnapshot()
      .then((current) => {
        if (!disposed) setSnapshot(current);
      })
      .catch((error: unknown) => {
        console.error("Could not read the replay buffer", error);
      });

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
    };
  }, []);

  const setReplayOn = (on: boolean) => {
    const change = on
      ? startReplayBuffer(startRecordingOptions())
      : stopReplayBuffer();
    change.catch((error: unknown) => {
      console.error("Could not switch the replay buffer", error);
    });
  };

  return { replay: snapshot, setReplayOn };
}
