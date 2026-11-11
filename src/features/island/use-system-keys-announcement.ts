// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

const ANNOUNCE_EVENT = "system-keys://announce";

type Announced = { keys: string; seen: boolean };

/**
 * The system's keys the island says once are now Screenwide's, told the
 * first time one of them opens a tool. Rust tells the island before that
 * tool's island is out, so it is held until an island that can say it
 * (`canSay`) has come out with it, and let go once that island tucks away.
 */
export function useSystemKeysAnnouncement(canSay: boolean) {
  const [announced, setAnnounced] = useState<Announced | null>(null);

  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    void listen<string>(ANNOUNCE_EVENT, ({ payload }) => {
      setAnnounced({ keys: payload, seen: false });
    }).then((unlisten) => {
      if (stopped) unlisten();
      else stop = unlisten;
    });
    return () => {
      stopped = true;
      stop?.();
    };
  }, []);

  // Marked while rendering, as the island that says it comes out.
  if (announced && !announced.seen && canSay)
    setAnnounced({ ...announced, seen: true });

  return {
    keys: announced?.keys ?? null,
    onHidden: () => {
      if (announced?.seen) setAnnounced(null);
    },
  };
}
