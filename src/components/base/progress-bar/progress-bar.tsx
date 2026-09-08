// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import {
  ProgressBar as AriaProgressBar,
  ProgressBarProps as AriaProgressBarProps,
} from "react-aria-components";

import { cn } from "../../../lib/styling";

/** One sweep of the indeterminate segment, in seconds. */
const SWEEP_DURATION = 1.25;

type ProgressBarProps = Omit<AriaProgressBarProps, "children" | "className"> & {
  className?: string;
};

/**
 * The native linear progress indicator: a capsule track with an accent fill,
 * matching an NSProgressIndicator bar and a WinUI ProgressBar.
 */
export function ProgressBar({
  className,
  isIndeterminate = false,
  ...props
}: ProgressBarProps) {
  const prefersReducedMotion = useReducedMotion();

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
          // The segment crosses the track and re-enters from the left; with
          // reduced motion it rests at the start and breathes in place.
          prefersReducedMotion ? (
            <span className="absolute inset-y-0 left-0 w-1/3 animate-pulse rounded-full bg-primary-surface" />
          ) : (
            <motion.span
              animate={{ left: ["-33%", "100%"] }}
              className="absolute inset-y-0 w-1/3 rounded-full bg-primary-surface"
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
            className="absolute inset-y-0 left-0 rounded-full bg-primary-surface"
            initial={false}
            transition={{ duration: 0.1, ease: "easeOut" }}
          />
        )
      }
    </AriaProgressBar>
  );
}

export type { ProgressBarProps };
