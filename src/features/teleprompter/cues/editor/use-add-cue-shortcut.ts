// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect, useRef } from "react";

import { ownsTextEditingKeys } from "../../../editor/shortcuts/keyboard-target";

/** ⌥C, or Alt+C: the shortcut that adds what an editor has open to a script
 * as a cue, by the key's place, whatever letter the layout puts there. */
export const ADD_CUE_SHORTCUT = { altKey: true, code: "KeyC" } as const;

/**
 * Runs `onAddCue` as ⌥C (Alt+C) is pressed in this window, where no text is
 * being typed. Its own listener rather than one more key in the editor's
 * shortcut table, which only knows ⌘ and Shift: ⌥ is otherwise left to the
 * pointer, as Option-drag duplicating. A tool panel forwards it here as it
 * does the editor's other keys.
 */
export function useAddCueShortcut(onAddCue: (() => void) | undefined) {
  const onAddCueRef = useRef(onAddCue);
  onAddCueRef.current = onAddCue;
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const onAdd = onAddCueRef.current;
      if (
        !onAdd ||
        event.code !== ADD_CUE_SHORTCUT.code ||
        !event.altKey ||
        event.ctrlKey ||
        event.metaKey ||
        event.shiftKey ||
        event.repeat ||
        event.isComposing ||
        ownsTextEditingKeys(event.target)
      )
        return;
      event.preventDefault();
      event.stopPropagation();
      onAdd();
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, []);
}
