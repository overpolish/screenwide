// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import {
  AnimatePresence,
  motion,
  MotionProps,
  useReducedMotion,
} from "motion/react";
import { ReactNode } from "react";

import { motionDurations, motionEasings } from "../../lib/motion";
import { cn } from "../../lib/styling";
import { Button } from "../../primitives/button/button";
import { IconButton } from "../../primitives/button/icon-button";

import { useConfirmAction } from "./use-confirm-action";

// A state change crossfades; native controls do not scale their content.
const SWAP_ANIMATION: MotionProps = {
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  initial: { opacity: 0 },
};

// Armed, the button is the primary button, as anything on is filled, so
// "press again to confirm" reads at a glance in either form, without a
// status colour on a label. Button draws it, hover and press included.
const ARMED = { color: "primary", variant: "solid" } as const;

type ConfirmActionButtonProps = {
  /** What the second press does, said while armed and shown on the text
   * button. */
  confirmLabel: string;
  /** What the first press offers to do. */
  label: string;
  className?: string;
  /** The glyph at rest; given, the button is an icon button. */
  icon?: ReactNode;
  /** Starts armed, for a story to show the armed look. */
  isArmedAtFirst?: boolean;
  isDisabled?: boolean;
  onConfirm?: () => void;
  timeoutMs?: number;
  variant?: "ghost" | "solid";
};

// An action that asks twice: the first press arms it, the second within a
// moment does it. Leaving the button, Escape, or the moment passing disarms
// it. With an icon it is an icon button whose glyph gives way to a check;
// without one, a text button whose label gives way to its confirm label,
// reserving the wider of the two words so arming never moves the layout.
export function ConfirmActionButton({
  className,
  confirmLabel,
  icon,
  isArmedAtFirst,
  isDisabled,
  label,
  onConfirm,
  timeoutMs,
  variant = "solid",
}: ConfirmActionButtonProps) {
  const reducedMotion = useReducedMotion();
  const { buttonProps, isArmed } = useConfirmAction({
    isArmedAtFirst,
    isDisabled,
    onConfirm,
    timeoutMs,
  });
  const { className: focusClassName, ...pressProps } = buttonProps;
  const transition = {
    duration: reducedMotion ? 0 : motionDurations.state,
    ease: motionEasings.out,
  };
  const buttonClassName = cn(className, focusClassName);
  const look = isArmed ? ARMED : { variant };
  const swap = (key: string, content: ReactNode) => (
    <AnimatePresence initial={false}>
      <motion.span
        aria-hidden
        className="col-start-1 row-start-1 flex items-center justify-center"
        key={key}
        {...SWAP_ANIMATION}
        transition={transition}
      >
        {content}
      </motion.span>
    </AnimatePresence>
  );

  if (icon === undefined) {
    return (
      <Button
        {...pressProps}
        aria-label={isArmed ? confirmLabel : label}
        className={buttonClassName}
        isDisabled={isDisabled}
        {...look}
      >
        {/* Both words hold the same cell, so the button is as wide as the
            wider one whichever is showing. */}
        <span className="grid">
          <span aria-hidden className="invisible col-start-1 row-start-1">
            {label}
          </span>
          <span aria-hidden className="invisible col-start-1 row-start-1">
            {confirmLabel}
          </span>
          {swap(isArmed ? "armed" : "idle", isArmed ? confirmLabel : label)}
        </span>
      </Button>
    );
  }

  return (
    <IconButton
      {...pressProps}
      aria-label={isArmed ? confirmLabel : label}
      className={buttonClassName}
      isDisabled={isDisabled}
      {...look}
    >
      <span className="grid">
        {swap(isArmed ? "armed" : "idle", isArmed ? <Check /> : icon)}
      </span>
    </IconButton>
  );
}

export type { ConfirmActionButtonProps };
