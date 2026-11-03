// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate, useMotionValue, useReducedMotion } from "motion/react";
import { useEffect, useLayoutEffect, useState } from "react";

import { closesAfter, contentIn, contentOut, glide } from "./island-motion";

/**
 * The island coming out and going away, as two steps: the black and what it
 * holds never move together. Out, the black grows empty and what it holds
 * fades in where it stands once the black is nearly there (`contentIn`).
 * Away, what it holds fades out where it stands (`contentOut`) and only
 * then does the black close (`isShown` turns false `closesAfter` later).
 * Asked back while still out, what it holds fades straight back in.
 *
 * `isShown` is whether the island is laid out as out, which everything
 * shaping the black follows; `contentOpacity` is what it holds; `isComing`
 * runs from the moment the black starts out or in until it has had the
 * black's glide to get there. Timed rather than told by the row's
 * animation finishing, which Motion reports for whichever change ends
 * first, often a frame in, while the black has barely started.
 */
export function useIslandPresence(isAsked: boolean) {
  const reduce = useReducedMotion() ?? false;
  const [isShown, setIsShown] = useState(isAsked);
  // The black grows from closed, so what it holds waits for it; mounted
  // shown, it grows from nothing once what it holds is measured.
  const [grows, setGrows] = useState(isAsked);
  if (isAsked && !isShown) {
    setIsShown(true);
    setGrows(true);
  }
  if (!isAsked && grows) setGrows(false);
  const [wasShown, setWasShown] = useState(isShown);
  // Mounted shown, it comes out from nothing as much as when shown later.
  const [isComing, setIsComing] = useState(isAsked);
  if (wasShown !== isShown) {
    setWasShown(isShown);
    setIsComing(true);
  }
  useEffect(() => {
    if (!isComing) return;
    const settled = window.setTimeout(
      () => {
        setIsComing(false);
      },
      reduce ? 0 : glide.duration * 1000 + 50,
    );
    return () => {
      window.clearTimeout(settled);
    };
  }, [isComing, isShown, reduce]);
  const contentOpacity = useMotionValue(0);
  useLayoutEffect(() => {
    if (isAsked) {
      const fading = animate(
        contentOpacity,
        1,
        reduce
          ? { duration: 0 }
          : grows
            ? contentIn
            : { ...contentIn, delay: 0 },
      );
      return () => {
        fading.stop();
      };
    }
    const fading = animate(
      contentOpacity,
      0,
      reduce ? { duration: 0 } : contentOut,
    );
    const closing = window.setTimeout(
      () => {
        setIsShown(false);
      },
      reduce ? 0 : closesAfter * 1000,
    );
    return () => {
      fading.stop();
      window.clearTimeout(closing);
    };
  }, [contentOpacity, grows, isAsked, reduce]);
  return { contentOpacity, isComing, isShown };
}
