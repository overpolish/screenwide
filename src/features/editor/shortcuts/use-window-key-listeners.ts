// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useEffect } from "react";

/** A shortcut hook's key handlers, as its latest render made them. */
export type WindowKeyHandlers = {
  onKeyDown: (event: KeyboardEvent) => void;
  onKeyUp: (event: KeyboardEvent) => void;
};

/**
 * The window's key listeners for one shortcut hook, added once and running
 * the handlers its latest render made. Added afresh whenever a handler
 * changed, a listener could be taken off mid-dispatch, when another hook's
 * shortcut re-renders the window between two listeners, and miss the key it
 * was for (⌘D letting go of the strip's choice but not the open mode).
 */
export function useWindowKeyListeners(
  handlersRef: RefObject<WindowKeyHandlers | null>,
  focusIntentRef: RefObject<"keyboard" | "pointer">,
) {
  useEffect(() => {
    const onPointerDown = () => {
      focusIntentRef.current = "pointer";
    };
    const onKeyDown = (event: KeyboardEvent) => {
      handlersRef.current?.onKeyDown(event);
    };
    const onKeyUp = (event: KeyboardEvent) => {
      handlersRef.current?.onKeyUp(event);
    };
    window.addEventListener("pointerdown", onPointerDown, true);
    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown, true);
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
    };
  }, [handlersRef, focusIntentRef]);
}
