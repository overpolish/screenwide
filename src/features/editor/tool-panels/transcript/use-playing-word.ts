// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { useEffect, useRef } from "react";

/** After you scroll the transcript yourself, how long it stays where you put
 * it before following the playhead again. */
const FOLLOW_AFTER_SCROLL_MS = 3_000;

/**
 * Marks what is being said, outside React, so a playhead moving every frame
 * touches a few elements rather than drawing every word again, and keeps
 * the first of them in view while nothing is chosen and you have not just
 * scrolled away. `elementsOf` finds what stands for `current`, -1 being
 * nothing. Returns what tells it you scrolled.
 */
export function usePlayingWord(
  elementsOf: (current: number) => readonly HTMLElement[],
  {
    content,
    current,
    isChoosing,
  }: { content: unknown; current: number; isChoosing: boolean },
) {
  const scrolledAtRef = useRef(Number.NEGATIVE_INFINITY);
  const followedRef = useRef(-1);
  const elementsOfRef = useRef(elementsOf);
  elementsOfRef.current = elementsOf;
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    const elements = current < 0 ? [] : elementsOfRef.current(current);
    if (elements.length === 0) return;
    for (const element of elements)
      element.toggleAttribute("data-current", true);
    if (
      followedRef.current !== current &&
      !isChoosing &&
      performance.now() - scrolledAtRef.current > FOLLOW_AFTER_SCROLL_MS
    )
      elements[0].scrollIntoView({
        behavior: reducedMotion ? "instant" : "smooth",
        block: "nearest",
      });
    followedRef.current = current;
    return () => {
      for (const element of elements)
        element.toggleAttribute("data-current", false);
    };
  }, [content, current, isChoosing, reducedMotion]);

  return () => {
    scrolledAtRef.current = performance.now();
  };
}
