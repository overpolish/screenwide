// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { DEFAULT_CONFIRM_TIMEOUT_MS } from "../confirm-action-button/use-confirm-action";

import type { ToggleButtonProps } from "react-aria-components";

type KeyDown = NonNullable<ToggleButtonProps["onKeyDown"]>;

/** The tile beside `tile` in its group, after it or else before it, which
 * takes the focus from a tile about to go. */
const neighbour = (tile: EventTarget) => {
  if (!(tile instanceof HTMLElement)) return null;
  const group = tile.closest('[role="radiogroup"], [role="toolbar"]');
  if (!group) return null;
  const tiles = [...group.querySelectorAll<HTMLElement>("button")];
  const at = tiles.indexOf(tile);
  if (at + 1 < tiles.length) return tiles[at + 1];
  return at > 0 ? tiles[at - 1] : null;
};

/**
 * Removal by two right clicks on the same tile, or two presses of Backspace
 * or Delete while it has the focus: the first arms it, the second removes
 * it.
 *
 * The rules are `useConfirmAction`'s, which is built around a button's own
 * press and focus rather than one tile of many: the armed window lapses,
 * Escape, leaving the tile or moving the focus off it disarm it, and arming
 * another tile disarms the first. A removal cannot be undone, so one stray
 * click or key must not be enough.
 */
export function useArmedRemoval(onRemove: (id: string) => void) {
  const [armed, setArmed] = useState<string | null>(null);
  const timerRef = useRef<number | undefined>(undefined);

  const disarm = () => {
    window.clearTimeout(timerRef.current);
    setArmed(null);
  };

  useEffect(() => {
    if (armed === null) return;
    // Captured, so an armed tile takes Escape before the editor's own
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

  /** A right click on `id`: arms it, or removes it when it is armed. */
  const secondary = (id: string) => {
    window.clearTimeout(timerRef.current);
    if (armed === id) {
      setArmed(null);
      onRemove(id);
      return;
    }
    setArmed(id);
    timerRef.current = window.setTimeout(() => {
      setArmed(null);
    }, DEFAULT_CONFIRM_TIMEOUT_MS);
  };

  return {
    armed,
    disarm,
    /** The keys on a focused tile: Backspace or Delete as a right click.
     * The focus moves to the tile beside one removed, so it is not lost. */
    keyboard: (id: string): { onBlur: () => void; onKeyDown: KeyDown } => ({
      onBlur: disarm,
      onKeyDown: (event) => {
        if (event.key !== "Backspace" && event.key !== "Delete") {
          event.continuePropagation();
          return;
        }
        event.preventDefault();
        const next = armed === id ? neighbour(event.target) : null;
        secondary(id);
        next?.focus();
      },
    }),
    secondary,
  };
}
