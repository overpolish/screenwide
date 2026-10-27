// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { HOVER_LINGER_MS } from "./island-motion";

/** The pointer coming onto the island and leaving it, for the island's own
 * box: leaving is told a moment after it happens, so a slip off the edge
 * does not close what hovering opened. */
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
  return {
    onPointerEnter: () => {
      window.clearTimeout(closeRef.current);
      onHoverChange?.(true);
    },
    onPointerLeave: () => {
      closeRef.current = window.setTimeout(() => {
        onHoverChange?.(false);
      }, HOVER_LINGER_MS);
    },
  };
}
