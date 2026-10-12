// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { RefObject, useEffect, useRef } from "react";

/** After you scroll the transcript yourself, how long it stays where you put
 * it before following the playhead again. */
const FOLLOW_AFTER_SCROLL_MS = 3_000;

/**
 * Marks the unit being said, outside React, so a playhead moving every frame
 * touches two elements rather than drawing every word again, and keeps it
 * in view while nothing is chosen and you have not just scrolled away.
 * Returns what tells it you scrolled.
 */
export function usePlayingWord(
  spansRef: RefObject<readonly HTMLElement[]>,
  {
    content,
    current,
    isChoosing,
  }: { content: unknown; current: number; isChoosing: boolean },
) {
  const scrolledAtRef = useRef(Number.NEGATIVE_INFINITY);
  const followedRef = useRef(-1);
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    const element = spansRef.current[current] as HTMLElement | undefined;
    if (!element) return;
    element.toggleAttribute("data-current", true);
    if (
      followedRef.current !== current &&
      !isChoosing &&
      performance.now() - scrolledAtRef.current > FOLLOW_AFTER_SCROLL_MS
    )
      element.scrollIntoView({
        behavior: reducedMotion ? "instant" : "smooth",
        block: "nearest",
      });
    followedRef.current = current;
    return () => {
      element.toggleAttribute("data-current", false);
    };
  }, [content, current, isChoosing, reducedMotion, spansRef]);

  return () => {
    scrolledAtRef.current = performance.now();
  };
}
