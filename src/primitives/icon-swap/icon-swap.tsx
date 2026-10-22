// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";

import { motionDurations, motionEasings } from "../../lib/motion";
import { cn } from "../../lib/styling";

type IconSwapProps = {
  children: ReactNode;
  /** Changes whenever the glyph does. */
  swapKey: string;
  className?: string;
};

// A glyph that changes is crossfaded where it sits: the old one shrinks and
// blurs away over the new one coming into focus, so nothing around it
// shifts. It fills its parent, which must be positioned and sized, so the two
// overlap exactly. The first glyph is shown as it is; with reduced motion the
// two only cross-fade.
export function IconSwap({ children, className, swapKey }: IconSwapProps) {
  const reducedMotion = useReducedMotion() ?? false;
  const away = reducedMotion
    ? { opacity: 0 }
    : { filter: "blur(2px)", opacity: 0, scale: 0.6 };
  return (
    <AnimatePresence initial={false}>
      <motion.span
        animate={{ filter: "blur(0px)", opacity: 1, scale: 1 }}
        className={cn(
          "absolute inset-0 flex items-center justify-center",
          className,
        )}
        exit={away}
        initial={away}
        key={swapKey}
        transition={{
          duration: motionDurations.travel,
          ease: motionEasings.out,
        }}
      >
        {children}
      </motion.span>
    </AnimatePresence>
  );
}
