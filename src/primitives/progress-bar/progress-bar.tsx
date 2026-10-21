// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import {
  ProgressBar as AriaProgressBar,
  ProgressBarProps as AriaProgressBarProps,
} from "react-aria-components";

import {
  progressPercent,
  useProgressTransition,
} from "../../lib/progress-transition";
import { cn } from "../../lib/styling";

/** One sweep of the indeterminate segment, in seconds. */
const SWEEP_DURATION = 1.25;

// A 6px capsule on the fill, a height only this bar needs, with the value in
// the primary colour, as all progress is. A value eases to each new figure;
// work that cannot say how far it has come sweeps a third-length segment
// across instead.
const fillClassName = "absolute inset-y-0 rounded-full bg-primary";

type ProgressBarProps = Omit<AriaProgressBarProps, "children" | "className"> & {
  className?: string;
};

export function ProgressBar({
  className,
  isIndeterminate = false,
  ...props
}: ProgressBarProps) {
  const prefersReducedMotion = useReducedMotion();
  const transition = useProgressTransition(
    progressPercent(props.value, props.minValue, props.maxValue),
  );

  return (
    <AriaProgressBar
      className={cn(
        "relative h-1.5 w-full overflow-hidden rounded-full bg-fill",
        className,
      )}
      isIndeterminate={isIndeterminate}
      {...props}
    >
      {({ percentage }) =>
        isIndeterminate ? (
          // The segment crosses the track and re-enters from the start; with
          // reduced motion it rests at the start and breathes in place.
          prefersReducedMotion ? (
            <span
              className={cn(fillClassName, "start-0 w-1/3 animate-pulse")}
            />
          ) : (
            <motion.span
              animate={{ left: ["-33%", "100%"] }}
              className={cn(fillClassName, "w-1/3")}
              transition={{
                duration: SWEEP_DURATION,
                ease: "linear",
                repeat: Infinity,
              }}
            />
          )
        ) : (
          <motion.span
            animate={{ width: `${String(percentage ?? 0)}%` }}
            className={cn(fillClassName, "start-0")}
            initial={false}
            transition={transition}
          />
        )
      }
    </AriaProgressBar>
  );
}

export type { ProgressBarProps };
