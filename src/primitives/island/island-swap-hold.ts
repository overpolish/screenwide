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
// closes in, what the section holds is not carried along: the part going
// stays where it was on the screen while it blurs away, the part coming is
// put where it will rest while it comes into focus, and the black glides
// round both. Where the content sits on the screen is the island's heading
// (where it is going sideways under a notch, which `sectionX` holds the
// section to), plus the box's place from the island's middle, plus the
// offset made here, so the offset is read every frame against both and the
// sum stays put. It hands from the old place to the new as the part going
// finishes leaving, and lets go when the box arrives. A move with no swap
// behind it, such as the row growing for controls under the pointer,
// carries what the section holds along as before.

export function useSwapHold({
  boxStart,
  heading,
  isOpen,
  isOpening,
  target,
}: {
  /** The box's place from the island's middle, as it is animated. */
  boxStart: MotionValue<string>;
  heading: MotionValue<number>;
  isOpen: boolean;
  isOpening: boolean;
  /** Where the box is going. */
  target: number;
}) {
  const swappedAtRef = useRef(Number.NEGATIVE_INFINITY);
  const previousRef = useRef(target);
  /** Where the content is held on the screen, from the island's resting
   * middle; none while it rides with the box. */
  const holdAt = useMotionValue(Number.NaN);
  const offset = useTransform(() => {
    const at = holdAt.get();
    return px(
      Number.isNaN(at) ? 0 : at - heading.get() - parseFloat(boxStart.get()),
    );
  });

  useLayoutEffect(() => {
    const from = previousRef.current;
    previousRef.current = target;
    const isSwap = performance.now() - swappedAtRef.current < SWAP_WINDOW_MS;
    if (from === target || !isOpen || isOpening || !isSwap) return;
    holdAt.set(heading.get() + parseFloat(boxStart.get()));
    // Read when it is due: by then the island knows where it is heading.
    const handOver = window.setTimeout(() => {
      holdAt.set(heading.get() + target);
    }, motionDurations.state * 1000);
    const letGo = window.setTimeout(() => {
      holdAt.set(Number.NaN);
    }, motionDurations.travel * 1000);
    return () => {
      window.clearTimeout(handOver);
      window.clearTimeout(letGo);
      holdAt.set(Number.NaN);
    };
  }, [boxStart, heading, holdAt, isOpen, isOpening, target]);

  const noteSwap = useCallback(() => {
    swappedAtRef.current = performance.now();
  }, []);
  const context = useMemo(() => ({ noteSwap }), [noteSwap]);

  return { context, offset };
}
