// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

/**
 * How to put the Export island away, while it stands over this editor
 * window: closed, tucked away while it exports, or let go once it is done.
 *
 * Module state for the reason the Escape claims are: the shortcut hooks on
 * the window are siblings on one listener target, so each asks here at event
 * time. Keys the island's window forwards are replayed on this window, so
 * they meet the same rule.
 */
let putIslandAway: (() => void) | null = null;

/** Registers the way the Export island is put away, or `null` while it
 * stands over nothing. */
export function useExportIslandKeys(putAway: (() => void) | null) {
  useEffect(() => {
    putIslandAway = putAway;
    return () => {
      putIslandAway = null;
    };
  }, [putAway]);
}

/**
 * Whether the Export island holds this key press. Under it nothing is
 * edited, so the window answers no shortcut but those that look through
 * the export: Space plays and pauses, and Export (⌘E) puts the island away
 * as its button does. Escape puts it away too; the first hook to see Escape
 * does so, and the rest find it consumed.
 */
export function exportIslandOwnsKey(
  event: KeyboardEvent,
  consume: (event: KeyboardEvent) => void,
) {
  if (!putIslandAway) return false;
  if (event.code === "Escape") {
    if (!event.defaultPrevented) {
      consume(event);
      putIslandAway();
    }
    return true;
  }
  const isBare =
    !event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey;
  if (event.code === "Space" && isBare) return false;
  const isExport =
    event.code === "KeyE" &&
    (event.metaKey || event.ctrlKey) &&
    !event.altKey &&
    !event.shiftKey;
  return !isExport;
}
