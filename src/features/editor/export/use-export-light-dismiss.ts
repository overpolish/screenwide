// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

/** How long after the press that put the island away its click is held
 * back; a click later than this is a press of its own. */
const CLICK_WITHIN_MS = 1_000;

/**
 * Puts the Export island away with `putAway` on a press anywhere in the
 * editor window round the picture: its title bar, strip and transport,
 * which are set back under it. The island's own window takes presses over
 * the picture (`editor-island-panel.tsx`). The press that puts it away
 * reaches nothing else, its click included, so it never also acts on what
 * is under it; the title bar's Export button is left to toggle the island
 * itself, and while a GIF is made (`isGifRange`) the strip and the range
 * tool (`data-gif-range`) are left to draw its range. Null while a press
 * off the island must do nothing, as while it exports.
 */
export function useExportLightDismiss(
  putAway: (() => void) | null,
  isGifRange: boolean,
) {
  const putAwayRef = useRef(putAway);
  putAwayRef.current = putAway;
  const isActive = putAway !== null;
  useEffect(() => {
    if (!isActive) return;
    const swallow = (event: MouseEvent) => {
      event.preventDefault();
      event.stopPropagation();
    };
    const spared = isGifRange
      ? "[data-export-toggle], [data-gif-range]"
      : "[data-export-toggle]";
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (target instanceof Element && target.closest(spared)) return;
      swallow(event);
      // Kept past this effect, which putting the island away ends: it goes
      // with the click it holds back, or by itself a moment later.
      // eslint-disable-next-line @eslint-react/web-api-no-leaked-event-listener -- removed by `once` or the timer below, not by the effect.
      window.addEventListener("click", swallow, { capture: true, once: true });
      // eslint-disable-next-line @eslint-react/web-api-no-leaked-timeout -- it only removes the listener above, which must outlive the effect.
      window.setTimeout(() => {
        window.removeEventListener("click", swallow, { capture: true });
      }, CLICK_WITHIN_MS);
      putAwayRef.current?.();
    };
    window.addEventListener("pointerdown", onPointerDown, { capture: true });
    return () => {
      window.removeEventListener("pointerdown", onPointerDown, {
        capture: true,
      });
    };
  }, [isActive, isGifRange]);
}
