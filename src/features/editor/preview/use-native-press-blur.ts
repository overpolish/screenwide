// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import {
  NATIVE_POINTER_EVENT,
  type NativePointerEvent,
} from "../choice/use-select-hold";

/** A press on the native picture takes focus and any text selection away
 * from the page, as a press on the page itself would. */
export function useNativePressBlur() {
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void getCurrentWindow()
      .listen<NativePointerEvent>(NATIVE_POINTER_EVENT, ({ payload }) => {
        if (!payload.down) return;
        if (document.activeElement instanceof HTMLElement) {
          document.activeElement.blur();
        }
        window.getSelection()?.removeAllRanges();
      })
      .then((dispose) => {
        unlisten = dispose;
      });
    return () => unlisten?.();
  }, []);
}
