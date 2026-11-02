// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSyncExternalStore } from "react";

// The card the pointer is over, which lights its segment in the storage
// strip. Held outside the browser's state: the pointer crosses cards all
// the time, and as state it would draw every card again each time, when
// only the strip shows it.

export type PointedCard = {
  get: () => string | null;
  set: (file: string | null) => void;
  subscribe: (listener: () => void) => () => void;
};

export function createPointedCard(): PointedCard {
  let pointed: string | null = null;
  const listeners = new Set<() => void>();
  return {
    get: () => pointed,
    set: (file) => {
      if (file === pointed) return;
      pointed = file;
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

/** The card pointed at, drawing again only as it changes. */
export const usePointedCard = (card: PointedCard) =>
  useSyncExternalStore(card.subscribe, card.get);
