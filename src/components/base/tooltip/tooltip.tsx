// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx } from "clsx";
import {
  AnimatePresence,
  motion,
  MotionProps,
  useReducedMotion,
} from "motion/react";
import { use } from "react";
import {
  Tooltip as AriaTooltip,
  TooltipProps as AriaTooltipProps,
  TooltipTriggerStateContext,
} from "react-aria-components";

// A native tooltip: a small panel on the window colour with the label in the
// small text style, a control radius and a soft shadow. No arrow, no colour
// inversion, and it fades rather than springs.
const tooltipClassName =
  "rounded-control bg-content px-control-inset py-control text-subheadline text-content-fg shadow-md";

type TooltipProps = Omit<AriaTooltipProps, keyof MotionProps> &
  MotionProps & {
    children?: React.ReactNode;
    className?: string;
  };

const MotionAriaTooltip = motion.create(AriaTooltip);

export const Tooltip = ({
  children,
  className,
  offset = 8,
  ...props
}: TooltipProps) => {
  const prefersReducedMotion = useReducedMotion();
  const triggerState = use(TooltipTriggerStateContext);
  const isOpen = props.isOpen ?? triggerState?.isOpen ?? false;

  return (
    <AnimatePresence>
      {isOpen ? (
        <MotionAriaTooltip
          {...props}
          animate={{ opacity: 1 }}
          className={clsx(tooltipClassName, className)}
          exit={{ opacity: 0 }}
          initial={{ opacity: 0 }}
          isOpen
          offset={offset}
          transition={{ duration: prefersReducedMotion ? 0 : 0.15 }}
        >
          {children}
        </MotionAriaTooltip>
      ) : null}
    </AnimatePresence>
  );
};
