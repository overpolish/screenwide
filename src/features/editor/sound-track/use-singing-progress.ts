// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useSyncExternalStore } from "react";

// How far the singing in each song is separated, as the app tells every
// window while it works (`editor://singing`). A song drops out once its
// work is done, or given up.

type SingingProgress = {
  /** From 0 to 1; null once the work is done or given up. */
  fraction: number | null;
  sound: string;
};

const fractions = new Map<string, number>();
const listeners = new Set<() => void>();
let isListening = false;

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

/** How far the singing in `sound` is separated, or null while no work on it
 * is under way. */
export function useSingingProgress(sound: string | null) {
  useEffect(() => {
    if (isListening) return;
    isListening = true;
    void listen<SingingProgress>("editor://singing", ({ payload }) => {
      if (payload.fraction === null) fractions.delete(payload.sound);
      else fractions.set(payload.sound, payload.fraction);
      for (const listener of listeners) listener();
    });
  }, []);
  return useSyncExternalStore(subscribe, () =>
    sound === null ? null : (fractions.get(sound) ?? null),
  );
}
