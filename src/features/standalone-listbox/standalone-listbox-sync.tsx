// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import {
  synchronizeStandaloneListboxStore,
  useStandaloneListboxStore,
} from "./store";

type StandaloneListboxClosed = {
  returnFocus: boolean;
  triggerId: string;
};

const restoreTriggerFocus = (triggerId: string) => {
  const trigger = [
    ...document.querySelectorAll<HTMLElement>(
      "[data-standalone-listbox-trigger]",
    ),
  ]
    .find((element) => element.dataset.standaloneListboxTrigger === triggerId)
    ?.querySelector<HTMLButtonElement>("button");
  trigger?.focus();
};

export function StandaloneListboxSync() {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    // The open listbox is presentation state and must not survive an app launch.
    useStandaloneListboxStore.getState().close();
    window.addEventListener("storage", synchronizeStandaloneListboxStore);
    void listen<StandaloneListboxClosed>(
      "standalone-listbox://closed",
      ({ payload }) => {
        useStandaloneListboxStore.getState().close();
        if (payload.returnFocus) {
          window.requestAnimationFrame(() => {
            restoreTriggerFocus(payload.triggerId);
          });
        }
      },
    ).then((listener) => {
      if (disposed) {
        listener();
      } else {
        unlisten = listener;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
      window.removeEventListener("storage", synchronizeStandaloneListboxStore);
    };
  }, []);

  return null;
}
