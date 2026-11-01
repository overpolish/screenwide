// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode, useState } from "react";

import { motionDurations, motionEasings } from "../../../../lib/motion";
import { cn } from "../../../../lib/styling";

const away = { height: 0, opacity: 0 };
const shown = { height: "auto", opacity: 1 };

/** A lane of the strip drawn only while it holds something (`children`, or
 * nothing). It grows from nothing as it arrives and collapses as it goes,
 * its content fading, so what is under it slides with it on the strip's
 * rearrange timing rather than jumping. */
export function StripLane({ children }: { children: ReactNode }) {
  const reducedMotion = useReducedMotion() ?? false;
  const isPresent = Boolean(children);
  // Clipped only while it grows or collapses: settled, what a lane draws a
  // little outside itself, such as a chosen annotation's handles, shows.
  const [wasPresent, setWasPresent] = useState(isPresent);
  const [isMoving, setIsMoving] = useState(false);
  if (isPresent !== wasPresent) {
    setWasPresent(isPresent);
    setIsMoving(true);
  }
  const settle = () => {
    setIsMoving(false);
  };
  return (
    <AnimatePresence initial={false} onExitComplete={settle}>
      {children ? (
        <motion.div
          animate={shown}
          className={cn(isMoving && "overflow-hidden")}
          // What leaves is drawn as it last was, so it is clipped here.
          exit={{ ...away, overflow: "hidden" }}
          initial={away}
          key="lane"
          onAnimationComplete={settle}
          transition={{
            duration: reducedMotion ? 0 : motionDurations.rearrange,
            ease: motionEasings.out,
          }}
        >
          {/* The gap above it is its own, inside what collapses, so it
              closes with it. */}
          <div className="pt-1">{children}</div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
