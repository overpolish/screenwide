// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx } from "clsx";
import {
  AnimatePresence,
  motion,
  MotionProps,
  useReducedMotion,
  Variants,
} from "motion/react";
import { use } from "react";
import {
  Tooltip as AriaTooltip,
  TooltipProps as AriaTooltipProps,
  OverlayArrow,
  TooltipTriggerStateContext,
} from "react-aria-components";

const tooltipClassName = [
  "bg-content-fg text-content m-2 px-control-inset py-control rounded-lg text-xs shadow-md",
  "data-[placement=top]:origin-bottom",
  "data-[placement=bottom]:origin-top",
  "data-[placement=left]:origin-right",
  "data-[placement=right]:origin-left",
];

const tooltipMotion: Variants = {
  closed: (placement: NonNullable<AriaTooltipProps["placement"]>) => ({
    opacity: 0,
    scale: 0.95,
    x: placement.startsWith("left")
      ? 4
      : placement.startsWith("right")
        ? -4
        : 0,
    y: placement.startsWith("top")
      ? 4
      : placement.startsWith("bottom")
        ? -4
        : 0,
  }),
  open: { opacity: 1, scale: 1, x: 0, y: 0 },
};

type TooltipProps = Omit<AriaTooltipProps, keyof MotionProps> &
  MotionProps & {
    children?: React.ReactNode;
    className?: string;
    withArrow?: boolean;
  };

const MotionAriaTooltip = motion.create(AriaTooltip);

export const Tooltip = ({
  arrowBoundaryOffset = 8,
  children,
  className,
  withArrow = true,
  ...props
}: TooltipProps) => {
  const prefersReducedMotion = useReducedMotion();
  const triggerState = use(TooltipTriggerStateContext);
  const isOpen = props.isOpen ?? triggerState?.isOpen ?? false;
  const placement = props.placement ?? "top";

  return (
    <AnimatePresence>
      {isOpen ? (
        <MotionAriaTooltip
          {...props}
          animate="open"
          arrowBoundaryOffset={arrowBoundaryOffset}
          className={clsx(tooltipClassName, className)}
          custom={placement}
          data-surface="inverse"
          exit="closed"
          initial="closed"
          isOpen
          transition={
            prefersReducedMotion
              ? { duration: 0 }
              : { damping: 25, stiffness: 300, type: "spring" }
          }
          variants={tooltipMotion}
        >
          {withArrow && (
            <OverlayArrow>
              {({ placement }) => {
                const resolvedPlacement = placement ?? props.placement ?? "top";
                return (
                  <svg
                    className={clsx(
                      "fill-content-fg",
                      (resolvedPlacement.startsWith("left") ||
                        resolvedPlacement.startsWith("start")) &&
                        "rotate-270",
                      (resolvedPlacement.startsWith("right") ||
                        resolvedPlacement.startsWith("end")) &&
                        "rotate-90",
                      resolvedPlacement.startsWith("bottom") && "rotate-180",
                    )}
                    height={8}
                    viewBox="0 0 8 8"
                    width={8}
                  >
                    <path d="M0 0 L4 4 L8 0" />
                  </svg>
                );
              }}
            </OverlayArrow>
          )}

          {children}
        </MotionAriaTooltip>
      ) : null}
    </AnimatePresence>
  );
};
