// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
  type: "pointerout" | "pointerover",
  {
    relatedTarget,
    x,
    y,
  }: { relatedTarget: Element | null; x: number; y: number },
) =>
  new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    clientX: x,
    clientY: y,
    composed: true,
    isPrimary: true,
    pointerId: 1,
    pointerType: "mouse",
    relatedTarget,
  });

/**
 * WebKit deliberately suppresses page hover while its macOS window is not
 * key, even though AppKit continues tracking the pointer. Native panel events
 * call this bridge with window-local coordinates so React Aria receives normal
 * pointer transitions without activating the recording UI.
 */
export function installInactiveWindowHoverBridge() {
  window.__SCREENWIDE_INACTIVE_HOVER__?.clear();

  let hovered: Element | null = null;
  const clear = () => {
    const previous = hovered;
    if (!previous) return;
    hovered = null;
    previous.dispatchEvent(
      pointerEvent("pointerout", { relatedTarget: null, x: 0, y: 0 }),
    );
  };

  window.__SCREENWIDE_INACTIVE_HOVER__ = {
    clear,
    move(x, yFromBottom) {
      const y = window.innerHeight - yFromBottom;
      const next = document.elementFromPoint(x, y);
      if (next === hovered) return;

      const previous = hovered;
      hovered = next;
      previous?.dispatchEvent(
        pointerEvent("pointerout", { relatedTarget: next, x, y }),
      );
      next?.dispatchEvent(
        pointerEvent("pointerover", { relatedTarget: previous, x, y }),
      );
    },
  };
}
