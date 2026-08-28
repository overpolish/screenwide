// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { ReactNode } from "react";
import {
  ProgressBar as AriaProgressBar,
  ProgressBarProps,
} from "react-aria-components";
import { VariantProps } from "tailwind-variants";

import { tv } from "../../../lib/variants";

const ANIMATION_DURATION = 1.25;

const circularProgressVariants = tv({
  defaultVariants: { size: "default" },
  slots: {
    backdrop: "stroke-neutral",
    base: "relative shrink-0",
    label:
      "absolute inset-0 flex items-center justify-center text-xl font-bold text-content-fg tabular-nums",
    progress: "stroke-primary-surface [stroke-linecap:round]",
  },
  variants: {
    size: {
      compact: { base: "size-4" },
      default: { base: "size-8" },
      large: { base: "size-24" },
    },
  },
});

type CircularProgressSize = NonNullable<
  VariantProps<typeof circularProgressVariants>["size"]
>;

const strokeWidths: Record<CircularProgressSize, number> = {
  compact: 10,
  default: 8,
  large: 8,
};

type CircularProgressProps = Omit<ProgressBarProps, "children" | "className"> &
  VariantProps<typeof circularProgressVariants> & {
    renderLabel?: (percentage?: number) => ReactNode;
  };

export function CircularProgress({
  isIndeterminate = false,
  renderLabel,
  size = "default",
  ...props
}: CircularProgressProps) {
  const resolvedSize = size;
  const { backdrop, base, label, progress } = circularProgressVariants({
    size: resolvedSize,
  });
  const strokeWidth = strokeWidths[resolvedSize];
  const radius = 50 - strokeWidth / 2;
  const circumference = 2 * Math.PI * radius;

  return (
    <AriaProgressBar
      className={base()}
      isIndeterminate={isIndeterminate}
      {...props}
    >
      {({ percentage }) => (
        <>
          <svg
            aria-hidden="true"
            className="size-full fill-none"
            strokeWidth={strokeWidth}
            viewBox="0 0 100 100"
          >
            <circle className={backdrop()} cx="50" cy="50" r={radius} />

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

          {renderLabel?.(percentage) ??
            (percentage !== undefined &&
            !isIndeterminate &&
            resolvedSize === "large" ? (
              <span className={label()}>{percentage.toFixed(0)}</span>
            ) : null)}
        </>
      )}
    </AriaProgressBar>
  );
}

export type { CircularProgressProps };
