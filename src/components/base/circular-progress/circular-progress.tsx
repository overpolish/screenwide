// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import {
  ProgressBar as AriaProgressBar,
  ProgressBarProps,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { tv } from "../../../lib/variants";

const ANIMATION_DURATION = 1.25;

const circularProgressVariants = tv({
  defaultVariants: { size: "regular" },
  slots: {
    base: "relative shrink-0",
    progress: "stroke-primary [stroke-linecap:round]",
    track: "stroke-fill",
  },
  variants: {
    size: {
      regular: { base: "size-8" },
      small: { base: "size-icon" },
    },
  },
});

type CircularProgressSize = NonNullable<
  VariantProps<typeof circularProgressVariants>["size"]
>;

/** Stroke widths in screen pixels, paired with the rendered box size. */
const strokeWidths: Record<CircularProgressSize, number> = {
  regular: 3,
  small: 2,
};

const boxSizes: Record<CircularProgressSize, number> = {
  regular: 32,
  small: 16,
};

type CircularProgressProps = Omit<ProgressBarProps, "children" | "className"> &
  VariantProps<typeof circularProgressVariants>;

export function CircularProgress({
  isIndeterminate = false,
  size = "regular",
  ...props
}: CircularProgressProps) {
  const { base, progress, track } = circularProgressVariants({ size });
  // The stroke is specified in screen pixels but drawn in viewBox units: a
  // non-scaling stroke would make WebKit apply the dash pattern in pixels too,
  // which breaks the pathLength normalisation the ring animates with.
  const strokeWidth = (strokeWidths[size] * 100) / boxSizes[size];
  const radius = 50 - strokeWidth / 2;
  const circumference = 2 * Math.PI * radius;

  return (
    <AriaProgressBar
      className={base()}
      isIndeterminate={isIndeterminate}
      {...props}
    >
      {({ percentage }) => (
        <svg
          aria-hidden="true"
          className="size-full fill-none"
          strokeWidth={strokeWidth}
          viewBox="0 0 100 100"
        >
          <circle className={track()} cx="50" cy="50" r={radius} />

          {isIndeterminate ? (
            <motion.circle
              animate={{
                rotate: [0, 180, 360],
                strokeDasharray: [
                  `${(circumference * 0.1).toString()} ${circumference.toString()}`,
                  `${(circumference * 0.25).toString()} ${circumference.toString()}`,
                  `${(circumference * 0.1).toString()} ${circumference.toString()}`,
                ],
                strokeDashoffset: [
                  circumference * 0.45,
                  circumference * 0.67,
                  circumference * 0.45,
                ],
              }}
              className={progress()}
              cx="50"
              cy="50"
              r={radius}
              style={{ transformOrigin: "50% 50%" }}
              transition={{
                rotate: {
                  duration: ANIMATION_DURATION,
                  ease: "linear",
                  repeat: Infinity,
                },
                strokeDasharray: {
                  duration: ANIMATION_DURATION,
                  ease: "easeInOut",
                  repeat: Infinity,
                },
                strokeDashoffset: {
                  duration: ANIMATION_DURATION,
                  ease: "easeInOut",
                  repeat: Infinity,
                },
              }}
            />
          ) : null}

          {percentage !== undefined && !isIndeterminate ? (
            <motion.circle
              animate={{ strokeDashoffset: 1 - percentage / 100 }}
              className={progress()}
              cx="50"
              cy="50"
              initial={false}
              pathLength={1}
              r={radius}
              strokeDasharray="1 1"
              transform="rotate(-90 50 50)"
              transition={{ duration: 0.1, ease: "easeOut" }}
            />
          ) : null}
        </svg>
      )}
    </AriaProgressBar>
  );
}

export type { CircularProgressProps };
