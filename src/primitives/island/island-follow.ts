// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Transition } from "motion/react";
import {
  createContext,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import { glide } from "./island-motion";

// The black glides to each size what it holds is measured at. A part
// folding its room away (`IslandParts`) changes that size every frame on the
// black's own curve, and a glide started afresh at every frame would trail
// it, the black closing well after what it holds. While a part folds, the
// black follows what it holds exactly instead, so the two close as one.

/** Has the island follow what it holds exactly for `seconds`. */
export const IslandFollowContext = createContext<(seconds: number) => void>(
  () => undefined,
);

const still: Transition = { duration: 0 };

/** How the island's black moves to a new size: gliding, or following what
 * it holds while asked to (`follow`). */
export function useIslandFollow() {
  const [isFollowing, setIsFollowing] = useState(false);
  const untilRef = useRef(0);
  const timerRef = useRef(0);
  const follow = useCallback((seconds: number) => {
    const until = performance.now() + seconds * 1000;
    if (until <= untilRef.current) return;
    untilRef.current = until;
    setIsFollowing(true);
    window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => {
      setIsFollowing(false);
    }, seconds * 1000);
  }, []);
  useEffect(
    () => () => {
      window.clearTimeout(timerRef.current);
    },
    [],
  );
  const travel: Transition = isFollowing ? still : glide;
  return { follow, travel };
}

/** `travel` as it is now, for an effect that moves the black when its
 * target changes: a change to `travel` alone moves nothing, so a glide
 * under way when following starts runs on to its end. */
export function useTravelRef(travel: Transition) {
  const travelRef = useRef(travel);
  useLayoutEffect(() => {
    travelRef.current = travel;
  });
  return travelRef;
}
