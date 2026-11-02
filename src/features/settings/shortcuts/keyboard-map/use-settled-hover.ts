// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { motionDurations } from "../../../../lib/motion";

// The key under the pointer, as the line under the map follows it. Moving
// to a neighbour hands the pointer straight over; leaving the map lets the
// line settle back to its hint a moment later, so it does not blink when
// the pointer crosses the map's edge on its way back in.

/** Reports the key under the pointer to `onHover`, and `null` a moment
 * after the pointer has left every key. Each key passes its hover changes
 * to what this returns. */
export function useSettledHover(onHover: (code: string | null) => void) {
  const currentRef = useRef<string | null>(null);
  const settleRef = useRef<number | undefined>(undefined);

  useEffect(
    () => () => {
      window.clearTimeout(settleRef.current);
    },
    [],
  );

  return (code: string, isHovered: boolean) => {
    if (isHovered) {
      window.clearTimeout(settleRef.current);
      currentRef.current = code;
      onHover(code);
      return;
    }
    // A neighbour's hover can arrive before this key's ends.
    if (currentRef.current !== code) return;
    window.clearTimeout(settleRef.current);
    settleRef.current = window.setTimeout(() => {
      currentRef.current = null;
      onHover(null);
    }, motionDurations.travel * 1000);
  };
}
