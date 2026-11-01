// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

/** The pointer came onto the native picture from the page around it. */
const POINTER_ARRIVED_EVENT = "preview://pointer-arrived";

/**
 * Marks the root `data-pointer-away` while the pointer is over the native
 * picture, which turns the `hover` variant off (`index.css`).
 *
 * The picture's input view lies over the webview, and WebKit drops a move
 * that lands over it, so a pointer carried straight from the strip onto the
 * picture never leaves the page as WebKit sees it: what it last lay over
 * stays hovered, and a later layout hovers whatever slides under that spot.
 * The view reports the pointer's arrival instead, and the page's next
 * pointer move, which comes as the pointer is back over the page, clears it.
 */
export function usePointerAway() {
  useEffect(() => {
    const root = document.documentElement;
    const back = () => {
      delete root.dataset.pointerAway;
    };
    let isDisposed = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWindow()
      .listen(POINTER_ARRIVED_EVENT, () => {
        root.dataset.pointerAway = "";
      })
      .then((dispose) => {
        if (isDisposed) dispose();
        else unlisten = dispose;
      });
    window.addEventListener("pointermove", back, true);
    return () => {
      isDisposed = true;
      unlisten?.();
      window.removeEventListener("pointermove", back, true);
      back();
    };
  }, []);
}
