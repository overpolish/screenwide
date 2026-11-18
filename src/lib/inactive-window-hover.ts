// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
type InactiveWindowHoverBridge = {
  clear: () => void;
  move: (x: number, yFromBottom: number) => void;
};

declare global {
  interface Window {
    __SCREENWIDE_INACTIVE_HOVER__?: InactiveWindowHoverBridge;
  }
}

const pointerEvent = (
  type: "pointermove" | "pointerout" | "pointerover",
  {
    movementX = 0,
    movementY = 0,
    relatedTarget = null,
    x,
    y,
  }: {
    x: number;
    y: number;
    movementX?: number;
    movementY?: number;
    relatedTarget?: Element | null;
  },
) =>
  new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    clientX: x,
    clientY: y,
    composed: true,
    isPrimary: true,
    movementX,
    movementY,
    pointerId: 1,
    pointerType: "mouse",
    relatedTarget,
  });

/**
 * WebKit deliberately suppresses page hover while its macOS window is not
 * key, even though AppKit continues tracking the pointer. Native panel events
 * call this bridge with window-local coordinates so React Aria receives normal
 * pointer transitions without activating the recording UI, and the page sees
 * the pointer move, as what waits for it to move on a part (the notes
 * island's controls) needs. WebKit does not set the cursor there either, so
 * the cursor the page styles under the pointer is handed to the window to
 * show (`inactive_cursor.rs`).
 */
export function installInactiveWindowHoverBridge() {
  window.__SCREENWIDE_INACTIVE_HOVER__?.clear();

  let hovered: Element | null = null;
  let last: { x: number; y: number } | null = null;
  let shownCursor: string | null = null;
  const showCursor = (cursor: string) => {
    if (cursor === shownCursor) return;
    shownCursor = cursor;
    invoke("show_inactive_cursor", { cursor }).catch(() => {
      // A window opened outside the app has no native side to show it.
    });
  };
  const clear = () => {
    const previous = hovered;
    shownCursor = null;
    last = null;
    if (!previous) return;
    hovered = null;
    previous.dispatchEvent(pointerEvent("pointerout", { x: 0, y: 0 }));
  };

  window.__SCREENWIDE_INACTIVE_HOVER__ = {
    clear,
    move(x, yFromBottom) {
      const y = window.innerHeight - yFromBottom;
      const next = document.elementFromPoint(x, y);
      showCursor(next ? getComputedStyle(next).cursor : "auto");
      const movement = last
        ? { movementX: x - last.x, movementY: y - last.y }
        : {};
      last = { x, y };
      if (next !== hovered) {
        const previous = hovered;
        hovered = next;
        previous?.dispatchEvent(
          pointerEvent("pointerout", { relatedTarget: next, x, y }),
        );
        next?.dispatchEvent(
          pointerEvent("pointerover", { relatedTarget: previous, x, y }),
        );
      }
      next?.dispatchEvent(pointerEvent("pointermove", { ...movement, x, y }));
    },
  };
}
