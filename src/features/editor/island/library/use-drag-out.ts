// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type PointerEvent, useRef } from "react";

/** How far, in px, a press moves before it becomes a drag. */
const DRAG_PX = 4;

/** Handlers for whatever wraps something in the library that can be pressed
 * and dragged out: a press that moves far enough starts `onDrag`, and one
 * that does not stays a press. They read the press before the button's own
 * handling takes it. None where nothing can be dragged out.
 *
 * The system's drag takes the release with it, so the press is called off
 * as the drag starts: the button lets it go rather than waiting for a
 * release, and a drag let go back over it is not a press. The window is
 * handed the release itself when the drag ends (`save/finished/macos.rs`). */
export function useDragOut(onDrag: (() => void) | undefined) {
  const pressRef = useRef<{ x: number; y: number } | null>(null);
  if (!onDrag) return {};
  return {
    onPointerDownCapture: (event: PointerEvent) => {
      if (event.button === 0)
        pressRef.current = { x: event.clientX, y: event.clientY };
    },
    onPointerMoveCapture: (event: PointerEvent) => {
      const press = pressRef.current;
      if (
        !press ||
        Math.hypot(event.clientX - press.x, event.clientY - press.y) < DRAG_PX
      )
        return;
      pressRef.current = null;
      event.target.dispatchEvent(
        new window.PointerEvent("pointercancel", {
          bubbles: true,
          isPrimary: event.isPrimary,
          pointerId: event.pointerId,
          pointerType: event.pointerType,
        }),
      );
      onDrag();
    },
    onPointerUpCapture: () => {
      pressRef.current = null;
    },
  };
}
