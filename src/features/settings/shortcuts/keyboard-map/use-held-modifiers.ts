// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import type { HotkeyModifier } from "../../../../lib/hotkey";

const MODIFIER_KEYS = ["Alt", "Control", "Meta", "Shift"];

// The real keyboard, read for the map: the modifiers held latch on the map
// as clicking them does, and stay latched when let go, so what they unlock
// can be read after the hand leaves the keys. Typing into a field is left
// alone, and a shortcut field that is recording keeps its keys to itself.
export function useHeldModifiers({
  onEscape,
  onHeld,
}: {
  onEscape: () => void;
  onHeld: (held: Set<HotkeyModifier>) => void;
}) {
  const callbacksRef = useRef({ onEscape, onHeld });
  useEffect(() => {
    callbacksRef.current = { onEscape, onHeld };
  });

  useEffect(() => {
    const read = (event: KeyboardEvent) => {
      if (
        event.defaultPrevented ||
        (event.target instanceof HTMLElement &&
          event.target.closest("input, textarea, [contenteditable]"))
      )
        return;
      if (event.key === "Escape") {
        callbacksRef.current.onEscape();
        return;
      }
      if (!MODIFIER_KEYS.includes(event.key)) return;
      const held = new Set<HotkeyModifier>();
      if (event.metaKey) held.add("Super");
      if (event.ctrlKey) held.add("Control");
      if (event.altKey) held.add("Alt");
      if (event.shiftKey) held.add("Shift");
      callbacksRef.current.onHeld(held);
    };
    window.addEventListener("keydown", read);
    return () => {
      window.removeEventListener("keydown", read);
    };
  }, []);
}
