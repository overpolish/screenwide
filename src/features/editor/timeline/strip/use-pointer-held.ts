// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

// Whether a pointer is pressed anywhere in the window, so what a drag
// changes, such as the squeeze or a trim, follows it live, while a change
// made with a press, a key or an undo is shown gliding. Read in the capture
// phase, so a button's press, which acts as the pointer is let go, already
// sees it let go.

export function usePointerHeld() {
  const heldRef = useRef(false);
  useEffect(() => {
    const press = () => {
      heldRef.current = true;
    };
    const release = () => {
      heldRef.current = false;
    };
    window.addEventListener("pointerdown", press, true);
    window.addEventListener("pointerup", release, true);
    window.addEventListener("pointercancel", release, true);
    window.addEventListener("blur", release);
    return () => {
      window.removeEventListener("pointerdown", press, true);
      window.removeEventListener("pointerup", release, true);
      window.removeEventListener("pointercancel", release, true);
      window.removeEventListener("blur", release);
    };
  }, []);
  return heldRef;
}
