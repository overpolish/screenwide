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
//
// Exported because the Editor draws its tooltips in a window of their own,
// over the native preview surface the page cannot reach. That window is built
// from these same classes, so the two cannot drift apart.
/** The tooltip's shape and type, shared with the native tooltip window. */
export const tooltipShapeClassName =
  "rounded-control px-control-inset py-control text-subheadline text-content-fg";

/** In the DOM a tooltip has no material behind it, so it paints its own
 * opaque panel; the native window shows the window material instead. */
const tooltipPanelClassName = `${tooltipShapeClassName} bg-content shadow-md`;

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
          className={clsx(tooltipPanelClassName, className)}
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
