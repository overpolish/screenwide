// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { openRecordingUi } from "../recording-sources/api";

import { getReplaySnapshot, startReplayBuffer, stopReplayBuffer } from "./api";
import { startRecordingOptions } from "./recording-request";
import { initialReplaySnapshot, ReplaySnapshot } from "./types";

const REPLAY_STATE_EVENT = "replay://state";
const REPLAY_ERROR_EVENT = "replay://error";
const REPLAY_START_REQUESTED_EVENT = "replay://start-requested";

const turnOn = () => startReplayBuffer(startRecordingOptions());

/** Shows the bar, whose warnings say what kept the buffer from starting. */
const revealBar = () => {
  openRecordingUi().catch((error: unknown) => {
    console.error("Could not show the recording bar", error);
  });
};

/**
 * The replay buffer as Rust reports it, and the switch the recording bar
 * turns it on and off with. Turning it on takes the bar's settings as they
 * are at that moment; later changes only set up the next recording.
 *
 * The tray turns it on through this window too, since only it holds those
 * settings. A tray start that cannot happen shows the bar rather than failing
 * out of sight; `canStart` is the bar's own readiness for its settings.
 */
export function useReplayBuffer(canStart: boolean) {
  const [snapshot, setSnapshot] = useState<ReplaySnapshot>(
    initialReplaySnapshot,
  );
  const canStartRef = useRef(canStart);
  useEffect(() => {
    canStartRef.current = canStart;
  }, [canStart]);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];
    let disposed = false;
    // A tray start still in flight. Rust reports a failed start before the
    // state that follows it, so the error always finds this still set.
    let trayStartPending = false;
    const failTrayStart = () => {
      if (!trayStartPending) return;
      trayStartPending = false;
      revealBar();
    };
    const keep = (listener: UnlistenFn) => {
      if (disposed) listener();
      else unlisteners.push(listener);
    };

    void listen<ReplaySnapshot>(REPLAY_STATE_EVENT, ({ payload }) => {
      setSnapshot(payload);
      if (payload.status !== "starting") trayStartPending = false;
    }).then(keep);
    // There is no toast surface; the switch follows the state Rust settles on.
    void listen<{ message: string }>(REPLAY_ERROR_EVENT, ({ payload }) => {
      console.error(`Replay buffer: ${payload.message}`);
      failTrayStart();
    }).then(keep);
    // Readiness only sees missing devices while the bar is visible, so a
    // start that passes it can still fail in Rust; both paths show the bar.
    void listen(REPLAY_START_REQUESTED_EVENT, () => {
      if (!canStartRef.current) {
        revealBar();
        return;
      }
      trayStartPending = true;
      turnOn().catch((error: unknown) => {
        console.error("Could not turn on the replay buffer", error);
        failTrayStart();
      });
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
    const change = on ? turnOn() : stopReplayBuffer();
    change.catch((error: unknown) => {
      console.error("Could not switch the replay buffer", error);
    });
  };

  return { replay: snapshot, setReplayOn };
}
