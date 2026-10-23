// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { useRecordingSourceStore } from "../recording-sources/store";

import { endScreenshotCapture } from "./screenshot-session";

const SCREENSHOT_DISMISS_REQUESTED_EVENT =
  "screenshot-region://dismiss-requested";

/** Escape, pressed here or handed over by Rust, ends a screenshot session
 * that has borrowed the overlay. */
export function useScreenshotCancel() {
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    let cancelling = false;
    const cancel = () => {
      if (!useRecordingSourceStore.getState().isScreenshotCapture) return;
      if (cancelling) return;
      cancelling = true;
      // Escape closes only the borrowed screenshot overlay. The ruler was
      // already open before this session and remains available underneath it.
      void endScreenshotCapture()
        .catch((error: unknown) => {
          console.error("Could not cancel the screenshot session", error);
        })
        .finally(() => {
          cancelling = false;
        });
    };
    const cancelKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") cancel();
    };

    window.addEventListener("keydown", cancelKey);
    void listen(SCREENSHOT_DISMISS_REQUESTED_EVENT, cancel).then((listener) => {
      if (disposed) listener();
      else unlisten = listener;
    });

    return () => {
      disposed = true;
      window.removeEventListener("keydown", cancelKey);
      unlisten?.();
    };
  }, []);
}
