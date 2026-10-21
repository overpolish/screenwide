// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { DEFAULT_CONFIRM_TIMEOUT_MS } from "../../composites/confirm-action-button/use-confirm-action";

/**
 * Removal by two right clicks on the same colour: the first arms it, the
 * second removes it.
 *
 * The rules are `useConfirmAction`'s, which is built around a button's own
 * press and focus rather than a right click on one tile of many: the armed
 * window lapses, Escape and leaving the tile disarm it, and arming another
 * colour disarms the first. A removal cannot be undone, so one stray right
 * click must not be enough.
 */
export function useArmedRemoval(onRemove: (color: string) => void) {
  const [armed, setArmed] = useState<string | null>(null);
  const timerRef = useRef<number | undefined>(undefined);

  const disarm = () => {
    window.clearTimeout(timerRef.current);
    setArmed(null);
  };

  useEffect(() => {
    if (armed === null) return;
    // Captured, so an armed colour takes Escape before the editor's own
    // Escape clears a selection or puts a tool down.
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      window.clearTimeout(timerRef.current);
      setArmed(null);
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [armed]);

  useEffect(
    () => () => {
      window.clearTimeout(timerRef.current);
    },
    [],
  );

  return {
    armed,
    disarm,
    /** A right click on `color`: arms it, or removes it when it is armed. */
    secondary: (color: string) => {
      window.clearTimeout(timerRef.current);
      if (armed === color) {
        setArmed(null);
        onRemove(color);
        return;
      }
      setArmed(color);
      timerRef.current = window.setTimeout(() => {
        setArmed(null);
      }, DEFAULT_CONFIRM_TIMEOUT_MS);
    },
  };
}
