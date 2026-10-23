// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { type ReactNode, type Ref, use, useLayoutEffect } from "react";

import { motionDurations, motionEasings } from "../../lib/motion";

import { IslandSectionContext } from "./island-context";
import { focusIn, reveal } from "./island-motion";

/** How the part leaving goes: blurred away before the black has finished
 * changing shape round it. */
const leaving = {
  duration: motionDurations.state,
  ease: motionEasings.out,
} as const;

/** One of the parts a section swaps between. The part leaving blurs away
 * where it was; the part arriving comes into focus where it will rest once
 * the other has gone, the way the row's halves are revealed, so a swap reads
 * as the glass clearing on something new rather than one set of controls cut
 * for another. Where the swap moves the section's box, the section holds
 * each part in its place while the black glides round them
 * (`island-swap-hold.ts`); arriving, a part tells it a swap is under way.
 *
 * Keyed inside `AnimatePresence` with `mode="popLayout"`: it takes a ref so
 * the outgoing part can be lifted from the flow while it goes, leaving only
 * the incoming one to size the island. */
export function IslandCrossfade({
  children,
  ref,
}: {
  children: ReactNode;
  ref?: Ref<HTMLDivElement>;
}) {
  const { noteSwap } = use(IslandSectionContext);
  useLayoutEffect(() => {
    noteSwap();
  }, [noteSwap]);
  return (
    <motion.div
      animate={reveal.animate}
      exit={{ ...focusIn.exit, transition: leaving }}
      initial={reveal.initial}
      ref={ref}
      transition={{ ...reveal.transition, delay: leaving.duration }}
    >
      {children}
    </motion.div>
  );
}
