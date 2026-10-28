// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";

import { motionDurations, motionEasings } from "../../../lib/motion";

/** The motion lines, each drawn from the circle's side outwards, so they
 * trail out behind it as it moves off and draw back into it as it stops. */
const LINES = ["M7 7H3", "M7 12H2", "M7 17H3"];

/** An annotation drawing itself in: a circle moved to the right with motion
 * lines trailing behind it, or, still, the circle alone at the centre, as an
 * annotation that appears whole. Changing between the two plays out: the
 * circle glides across and the lines draw in after it, one by one, or back
 * into it. Drawn on a Lucide icon's grid with its stroke, and carrying its
 * class, so it takes the sizes set for them. */
export function DrawInIcon({
  className,
  isMoving,
}: {
  isMoving: boolean;
  className?: string;
}) {
  const reducedMotion = useReducedMotion() ?? false;
  const glide = reducedMotion
    ? { duration: 0 }
    : { bounce: 0.25, type: "spring" as const, visualDuration: 0.3 };
  return (
    <svg
      aria-hidden
      className={className ? `lucide ${className}` : "lucide"}
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <motion.circle
        animate={{ cx: isMoving ? 16 : 12 }}
        cy={12}
        initial={false}
        r={6}
        transition={glide}
      />
      {LINES.map((d, index) => (
        <motion.path
          animate={
            isMoving
              ? { opacity: 1, pathLength: 1 }
              : { opacity: 0, pathLength: 0 }
          }
          d={d}
          initial={false}
          key={d}
          transition={
            reducedMotion
              ? { duration: 0 }
              : {
                  // In after the circle has started away, the middle one
                  // first; out together, quicker, as it comes to rest.
                  delay: isMoving ? 0.06 + Math.abs(index - 1) * 0.05 : 0,
                  duration: isMoving
                    ? motionDurations.travel
                    : motionDurations.state,
                  ease: motionEasings.out,
                }
          }
        />
      ))}
    </svg>
  );
}
