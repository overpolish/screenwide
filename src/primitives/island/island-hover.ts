// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { HOVER_LINGER_MS } from "./island-motion";

/** The pointer coming onto the island and leaving it, for the island's own
 * box: leaving is told a moment after it happens, so a slip off the edge
 * does not close what hovering opened. Moving on the island tells it the
 * pointer is on it too: the island's window ignores the pointer while it is
 * tucked away, so a pointer resting where it comes back out was never seen
 * to leave, and is never seen to enter. */
export function useIslandHover(
  onHoverChange: ((isHovered: boolean) => void) | undefined,
) {
  const closeRef = useRef(0);
  useEffect(
    () => () => {
      window.clearTimeout(closeRef.current);
    },
    [],
  );
  const enter = () => {
    window.clearTimeout(closeRef.current);
    onHoverChange?.(true);
  };
  return {
    onPointerEnter: enter,
    onPointerLeave: () => {
      closeRef.current = window.setTimeout(() => {
        onHoverChange?.(false);
      }, HOVER_LINGER_MS);
    },
    onPointerMove: enter,
  };
}
