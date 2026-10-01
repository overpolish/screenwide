// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

/**
 * How to close the sheet standing over this editor window, while one is up.
 *
 * Module state for the reason the Escape claims are: the shortcut hooks on
 * the window are siblings on one listener target, so each asks here at event
 * time. Keys the tool panel forwards are replayed on this window, so they meet
 * the same rule.
 */
let dismissSheet: (() => void) | null = null;

/** Registers the way out of the sheet that is up, or `null` while none is. */
export function useSheetEscape(dismiss: (() => void) | null) {
  useEffect(() => {
    dismissSheet = dismiss;
    return () => {
      dismissSheet = null;
    };
  }, [dismiss]);
}

/**
 * Whether a sheet owns this key press. Under a sheet the window answers no
 * shortcut, the way its content answers no pointer, and Escape closes the
 * sheet. The first hook to see Escape closes it; the rest find it consumed.
 */
export function sheetOwnsKey(
  event: KeyboardEvent,
  consume: (event: KeyboardEvent) => void,
) {
  if (!dismissSheet) return false;
  if (event.code === "Escape" && !event.defaultPrevented) {
    consume(event);
    dismissSheet();
  }
  return true;
}
