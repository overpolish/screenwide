// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { keyOfCode, type Modifier } from "./shortcut-model";

const PRESS_MS = 220;
const MODIFIER_KEYS = ["Alt", "AltGraph", "Control", "Meta", "Shift"];

// The real keyboard, read for the map: the modifiers held light their keys
// as clicking them on screen does, and a key pressed with them looks up the
// shortcut it is. Letting go keeps what was held, so a combination can be
// read after the hand leaves the keys. Typing into a field is left alone,
// and a shortcut field that is recording keeps its keys to itself.

export function useHeldKeys({
  onEscape,
  onHeldChange,
  onLookUp,
}: {
  onEscape: () => void;
  onHeldChange: (held: ReadonlySet<Modifier>) => void;
  /** Whether the key and modifiers are a shortcut, which then takes the
   * press from the browser. */
  onLookUp: (key: string, mods: ReadonlySet<Modifier>) => boolean;
}) {
  const [pressedKey, setPressedKey] = useState<string | null>(null);
  const callbacksRef = useRef({ onEscape, onHeldChange, onLookUp });
  useEffect(() => {
    callbacksRef.current = { onEscape, onHeldChange, onLookUp };
  });

  useEffect(() => {
    const read = (event: KeyboardEvent) => {
      if (
        event.target instanceof HTMLElement &&
        event.target.closest("input, textarea, [contenteditable]")
      )
        return;
      if (event.type === "keydown" && event.key === "Escape") {
        callbacksRef.current.onEscape();
        return;
      }
      const next = new Set<Modifier>();
      if (event.metaKey) next.add("cmd");
      if (event.ctrlKey) next.add("ctrl");
      if (event.altKey) next.add("opt");
      if (event.shiftKey) next.add("shift");
      if (event.type === "keydown" || next.size > 0)
        callbacksRef.current.onHeldChange(next);
      if (
        event.type !== "keydown" ||
        event.repeat ||
        MODIFIER_KEYS.includes(event.key)
      )
        return;
      const key = keyOfCode(event.code);
      setPressedKey(key);
      if (callbacksRef.current.onLookUp(key, next)) event.preventDefault();
    };
    window.addEventListener("keydown", read);
    window.addEventListener("keyup", read);
    return () => {
      window.removeEventListener("keydown", read);
      window.removeEventListener("keyup", read);
    };
  }, []);

  useEffect(() => {
    if (!pressedKey) return;
    const timer = window.setTimeout(() => {
      setPressedKey(null);
    }, PRESS_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [pressedKey]);

  return pressedKey;
}
