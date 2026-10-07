// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

const MOMENT_PLACED_EVENT = "moments://placed";

/** Long enough to read a kind's name at a glance, short enough that the
 * timer is back before the next moment is likely. */
const SHOWN_MS = 1_500;

export type PlacedMoment = {
  color: string;
  /** Tells two moments of one kind in a row apart. */
  key: number;
  name: string;
};

/** The moment a kind shortcut placed last, while the dock shows it. The
 * dock is the only feedback a moment gets: a sound would be heard in the
 * recording's system audio. */
export function usePlacedMoment() {
  const [moment, setMoment] = useState<PlacedMoment | null>(null);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    let hide = 0;
    let count = 0;
    void listen<{ color: string; name: string }>(
      MOMENT_PLACED_EVENT,
      ({ payload }) => {
        count += 1;
        setMoment({ ...payload, key: count });
        window.clearTimeout(hide);
        hide = window.setTimeout(() => {
          setMoment(null);
        }, SHOWN_MS);
      },
    ).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      window.clearTimeout(hide);
      unlisten?.();
    };
  }, []);

  return moment;
}
