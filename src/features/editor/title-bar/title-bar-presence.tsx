// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";

/** How a title bar part comes and goes, or at once under reduced motion. */
function usePresenceTransition() {
  const reduce = useReducedMotion();
  return reduce
    ? { duration: 0 }
    : { duration: motionDurations.rearrange, ease: motionEasings.out };
}

/** A title bar part that grows in from nothing and shrinks back out as it
 * comes and goes inside an `AnimatePresence`, faded as it goes, so its
 * neighbours slide over rather than jump. Spacing it stands apart by goes
 * inside it, so it leaves with the part. */
export function TitleBarPresence({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  const transition = usePresenceTransition();
  return (
    <motion.div
      animate={{ opacity: 1, width: "auto" }}
      // Clipped with room for a focus ring round what it holds.
      className={cn(
        "flex shrink-0 justify-center overflow-clip [overflow-clip-margin:3px]",
        className,
      )}
      exit={{ opacity: 0, width: 0 }}
      initial={{ opacity: 0, width: 0 }}
      transition={transition}
    >
      {children}
    </motion.div>
  );
}
