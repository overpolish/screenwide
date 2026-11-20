// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSyncExternalStore } from "react";

import { type View } from "../timeline-scale";

// The part of the recording the strip shows this moment. A pan moves it
// many times a second without the strip drawing again, since nothing on the
// track moves when it scrolls; what does follow it, the minimap's window,
// reads it here and draws on its own.

export type StripViewNow = {
  get: () => View;
  /** Holds `view` as the one shown now, telling whoever reads it. */
  hold: (view: View) => void;
  subscribe: (listener: () => void) => () => void;
};

export function stripViewNow(initial: View): StripViewNow {
  let held = initial;
  const listeners = new Set<() => void>();
  return {
    get: () => held,
    hold: (view) => {
      if (view.start === held.start && view.end === held.end) return;
      held = view;
      for (const listener of listeners) listener();
    },
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}

/** The view `now` holds, drawn again each time it changes. */
export const useStripViewNow = (now: StripViewNow) =>
  useSyncExternalStore(now.subscribe, now.get);
