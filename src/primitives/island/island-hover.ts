// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { use, useEffect, useRef } from "react";

import { HOVER_LINGER_MS } from "./island-motion";
import { IslandPointerContext } from "./island-pointer";

/** The pointer coming onto the island and leaving it, for the island's own
 * box: leaving is told a moment after it happens, so a slip off the edge
 * does not close what hovering opened. Moving on the island tells it the
 * pointer is on it too: the island's window ignores the pointer while it is
 * tucked away, so a pointer resting where it comes back out was never seen
 * to leave, and is never seen to enter.
 *
 * Where the window says whether the pointer is on the island
 * (`IslandPointerContext`), it decides: the page is told nothing once the
 * window lets the pointer through, so a leaving it missed would hold the
 * hover, and WebKit replays the last place it saw the pointer when the
 * island changes under it, which would start one. An island that goes while
 * hovered says the pointer left, as the hover belongs to whoever passed
 * `onHoverChange`, which may outlive it. */
export function useIslandHover(
  onHoverChange: ((isHovered: boolean) => void) | undefined,
) {
  const closeRef = useRef(0);
  const isHoveredRef = useRef(false);
  // Heard by the page before the window found the pointer on the island,
  // which it does a moment later when the pointer is really there.
  const isEarlyRef = useRef(false);
  const onHoverChangeRef = useRef(onHoverChange);
  useEffect(() => {
    onHoverChangeRef.current = onHoverChange;
  });
  useEffect(
    () => () => {
      window.clearTimeout(closeRef.current);
      if (isHoveredRef.current) onHoverChangeRef.current?.(false);
    },
    [],
  );
  const isPointerOn = use(IslandPointerContext);
  const hover = () => {
    window.clearTimeout(closeRef.current);
    isHoveredRef.current = true;
    onHoverChangeRef.current?.(true);
  };
  const enter = () => {
    if (isPointerOn === false) {
      isEarlyRef.current = true;
      return;
    }
    hover();
  };
  const leave = () => {
    isEarlyRef.current = false;
    window.clearTimeout(closeRef.current);
    closeRef.current = window.setTimeout(() => {
      isHoveredRef.current = false;
      onHoverChangeRef.current?.(false);
    }, HOVER_LINGER_MS);
  };
  // Only the window's report changing ends or confirms the hover; `leave`
  // and `hover` read the handler through a ref.
  useEffect(() => {
    if (isPointerOn === false) leave();
    else if (isPointerOn && isEarlyRef.current) {
      isEarlyRef.current = false;
      hover();
    }
  }, [isPointerOn]);
  return {
    onPointerEnter: enter,
    onPointerLeave: leave,
    onPointerMove: enter,
  };
}
