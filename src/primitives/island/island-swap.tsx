// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type CSSProperties, type ReactNode, type Ref, useState } from "react";

import { arrive, focusIn, leave, reveal } from "./island-motion";

// Leaves quickly, as a section's part does, so the two never read as laid
// over each other.
const leaving = { ...focusIn.exit, transition: leave };

// Swaps what a row or the line under it holds when `swapKey` changes: the one
// leaving blurs away, lifted out (`popLayout`) so only the new one sizes the
// island, and the new one is revealed once it has gone (`arrive`). What it
// first holds is revealed a beat after the black opens (`reveal`). A
// section's parts swap with `IslandCrossfade`, which also holds them while
// the section's box moves.
export function IslandSwap({
  children,
  className,
  initial = true,
  ref,
  style,
  swapKey,
}: {
  children: ReactNode;
  swapKey: string;
  className?: string;
  // Off, what it first holds is laid out in place and only revealed as the
  // black opens round it.
  /** Reveals what it first holds as it mounts. */
  initial?: boolean;
  ref?: Ref<HTMLDivElement>;
  style?: CSSProperties;
}) {
  // Set while rendering rather than in an effect, so the part that arrives
  // with the first change already waits for the one leaving.
  const [seenKey, setSeenKey] = useState(swapKey);
  const [hasSwapped, setHasSwapped] = useState(false);
  if (swapKey !== seenKey) {
    setSeenKey(swapKey);
    setHasSwapped(true);
  }
  return (
    <AnimatePresence initial={initial} mode="popLayout">
      <motion.div
        animate={reveal.animate}
        className={className}
        exit={leaving}
        initial={reveal.initial}
        key={swapKey}
        ref={ref}
        style={style}
        transition={
          hasSwapped || swapKey !== seenKey ? arrive : reveal.transition
        }
      >
        {children}
      </motion.div>
    </AnimatePresence>
  );
}
