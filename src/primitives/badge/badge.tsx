// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { ReactNode } from "react";

import { motionDurations, motionEasings } from "../../lib/motion";
import { cn } from "../../lib/styling";

type BadgeProps = {
  children: ReactNode;
  className?: string;
  /** Pops in and shrinks away as this changes. */
  isShown?: boolean;
};

// A small part: caption text on the faint fill, at a part's corner. Inline
// and a span, so it can sit in a label or a button.
// Tabular figures keep a count or a size from changing width as it changes.
// One that comes and goes is kept mounted and given `isShown`, so it can
// leave as well as arrive; one that is there from the start is drawn as it
// is, so a page of them does not pop on load. With reduced motion it only
// fades.
export function Badge({ children, className, isShown = true }: BadgeProps) {
  const reducedMotion = useReducedMotion() ?? false;
  const away = reducedMotion ? { opacity: 0 } : { opacity: 0, scale: 0.8 };
  return (
    <AnimatePresence initial={false}>
      {isShown ? (
        <motion.span
          animate={{ opacity: 1, scale: 1 }}
          className={cn(
            "inline-flex h-part-height shrink-0 items-center gap-inline rounded-part bg-fill px-inset-sm text-caption whitespace-nowrap text-fg-secondary tabular-nums",
            "[&_svg]:size-icon-sm [&_svg]:shrink-0 [&_svg]:transform-gpu",
            className,
          )}
          exit={{
            ...away,
            transition: {
              duration: motionDurations.state,
              ease: motionEasings.out,
            },
          }}
          initial={away}
          transition={{ bounce: 0.3, duration: 0.3, type: "spring" }}
        >
          {children}
        </motion.span>
      ) : null}
    </AnimatePresence>
  );
}
