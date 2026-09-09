// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { synchronizePopupPanelStore, usePopupPanelStore } from "./store";

type PopupPanelClosed = {
  returnFocus: boolean;
  triggerId: string;
};

const restoreTriggerFocus = (triggerId: string) => {
  const trigger = [
    ...document.querySelectorAll<HTMLElement>("[data-popup-panel-trigger]"),
  ]
    .find((element) => element.dataset.popupPanelTrigger === triggerId)
    ?.querySelector<HTMLButtonElement>("button");
  trigger?.focus();
};

export function PopupPanelSync() {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    // The open listbox is presentation state and must not survive an app launch.
    usePopupPanelStore.getState().close();
    window.addEventListener("storage", synchronizePopupPanelStore);
    void listen<PopupPanelClosed>(
      "standalone-listbox://closed",
      ({ payload }) => {
        usePopupPanelStore.getState().close();
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
      window.removeEventListener("storage", synchronizePopupPanelStore);
    };
  }, []);

  return null;
}
