// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { type ReactNode, type Ref, use, useLayoutEffect } from "react";

import { IslandSectionContext } from "./island-context";
import { arrive, focusIn, leave, reveal } from "./island-motion";

// One of the parts a section swaps between: the one leaving blurs away where
// it was, the one arriving comes into focus where it will rest, so a swap
// reads as the glass clearing rather than a cut. Where the swap moves the
// section's box, the section holds each part in place while the black glides
// round them (`island-swap-hold.ts`); arriving, a part tells it a swap is
// under way. Keyed inside `AnimatePresence` with `mode="popLayout"`, it takes
// a ref so the outgoing part leaves the flow and only the incoming one sizes
// the island.
export function IslandCrossfade({
  children,
  ref,
}: {
  children: ReactNode;
  ref?: Ref<HTMLDivElement>;
}) {
  const { noteSwap, offset } = use(IslandSectionContext);
  useLayoutEffect(() => {
    noteSwap();
  }, [noteSwap]);
  return (
    <motion.div
      animate={reveal.animate}
      exit={{ ...focusIn.exit, transition: leave }}
      initial={reveal.initial}
      ref={ref}
      style={offset ? { marginInlineStart: offset } : undefined}
      transition={arrive}
    >
      {children}
    </motion.div>
  );
}
