// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { hidePopupPanel } from "./api";

/**
 * The keys a panel window answers while it holds the keyboard, as Windows
 * gives it to a menu opened by a press, with nothing in it focused yet:
 * Escape puts the panel away, as a press outside it does, and an up or down
 * arrow steps into its list. A panel that never takes the keyboard, as on
 * macOS, hears none of them. Caught before the list's own handling, so
 * Escape inside the list puts the panel away rather than clearing a choice.
 */
export function usePanelKeys(panel: string) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        void hidePopupPanel(true, panel);
        return;
      }
      if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
      if (document.activeElement && document.activeElement !== document.body)
        return;
      const list = document.querySelector<HTMLElement>('[role="listbox"]');
      if (!list) return;
      event.preventDefault();
      list.focus();
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [panel]);
}
