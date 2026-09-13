// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import { openPopupPanels } from "./api";
import { synchronizePopupPanelStore, usePopupPanelStore } from "./store";

type PopupPanelClosed = {
  /** Which panel window closed. Only its own entry is cleared: the other
   * windows' panels are untouched by this one going away. */
  panel: string;
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

    // The open listbox is presentation state and must not survive an app
    // launch. Every window mounts this, and one created later, such as the
    // tooltip's on its first hover, must not put away panels the editor has
    // open; so only the recording bar, which boots with the app and is never
    // created lazily, does the reset. It keeps the panels Rust has on screen:
    // an editor recovered after a crash may have its tool panel up before
    // the bar has finished booting, and that panel is not a leftover.
    if (isTauri() && getCurrentWindow().label === "recording-bar") {
      void openPopupPanels().then((panels) => {
        usePopupPanelStore.getState().keepOnly(panels);
      });
    }
    window.addEventListener("storage", synchronizePopupPanelStore);
    void listen<PopupPanelClosed>(
      "standalone-listbox://closed",
      ({ payload }) => {
        usePopupPanelStore.getState().close(payload.panel);
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
