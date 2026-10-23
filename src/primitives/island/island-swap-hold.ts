// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type MotionValue, useMotionValue, useTransform } from "motion/react";
import { useCallback, useLayoutEffect, useMemo, useRef } from "react";

import { motionDurations } from "../../lib/motion";

import { px } from "./island-geometry";

/** How soon after a part comes in the section's box has to start moving for
 * the move to be taken as the swap's, in ms: the parts are measured a frame
 * or two after they mount, and the box is sent on from there. */
const SWAP_WINDOW_MS = 120;

// When one part is swapped for another and the section's box moves with it,
// as when the row narrows round a tool that drops a control and the island
// closes in from both sides, the parts swapping are not carried along: the
// part going stays where it was while it blurs away, the part coming is put
// where it will rest while it comes into focus, and the black glides round
// both. Anything else in the section, such as a row of tools above the
// parts, rides with the box as the island's row does. The hold is an offset
// against the box's own glide, read every frame from the box's place, so the
// sum stays put; it hands from the old place to the new as the part going
// finishes leaving, and lets go when the box arrives. A move with no swap
// behind it, such as the row growing for controls under the pointer,
// carries everything along as before. Around a notch nothing is held: what
// the section holds is already put where it will rest
// (`island-section-settle.ts`).

export function useSwapHold({
  boxStart,
  isEnabled,
  isOpen,
  isOpening,
  target,
}: {
  /** The box's place from the island's middle, as it is animated. */
  boxStart: MotionValue<string>;
  /** Off around a notch. */
  isEnabled: boolean;
  isOpen: boolean;
  isOpening: boolean;
  /** Where the box is going. */
  target: number;
}) {
  const swappedAtRef = useRef(Number.NEGATIVE_INFINITY);
  const previousRef = useRef(target);
  /** Where the parts are held, from the island's middle; none while they
   * ride with the box. */
  const holdAt = useMotionValue(Number.NaN);
  const offset = useTransform(() => {
    const at = holdAt.get();
    return px(Number.isNaN(at) ? 0 : at - parseFloat(boxStart.get()));
  });

  useLayoutEffect(() => {
    const from = previousRef.current;
    previousRef.current = target;
    const isSwap = performance.now() - swappedAtRef.current < SWAP_WINDOW_MS;
    if (from === target || !isEnabled || !isOpen || isOpening || !isSwap)
      return;
    holdAt.set(parseFloat(boxStart.get()));
    const handOver = window.setTimeout(() => {
      holdAt.set(target);
    }, motionDurations.state * 1000);
    const letGo = window.setTimeout(() => {
      holdAt.set(Number.NaN);
    }, motionDurations.travel * 1000);
    return () => {
      window.clearTimeout(handOver);
      window.clearTimeout(letGo);
      holdAt.set(Number.NaN);
    };
  }, [boxStart, holdAt, isEnabled, isOpen, isOpening, target]);

  const noteSwap = useCallback(() => {
    swappedAtRef.current = performance.now();
  }, []);
  return useMemo(() => ({ noteSwap, offset }), [noteSwap, offset]);
}
