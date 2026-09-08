// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnimatePresence,
  motion,
  MotionProps,
  useReducedMotion,
} from "motion/react";
import { ReactNode } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Button } from "../../base/button/button";
import { IconButton } from "../../base/button/icon-button";

import { useConfirmAction } from "./use-confirm-action";

// A state change crossfades; native controls do not scale their content.
const SWAP_ANIMATION: MotionProps = {
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  initial: { opacity: 0 },
};

type ConfirmActionButtonProps = {
  armedIcon: ReactNode;
  armedLabel: string;
  idleIcon: ReactNode;
  idleLabel: string;
  className?: string;
  isDisabled?: boolean;
  onConfirm?: () => void;
  timeoutMs?: number;
  /** Icon-only, or an icon beside the label it is confirming. */
  variant?: "icon" | "text";
};

export function ConfirmActionButton({
  armedIcon,
  armedLabel,
  className,
  idleIcon,
  idleLabel,
  isDisabled,
  onConfirm,
  timeoutMs,
  variant = "icon",
}: ConfirmActionButtonProps) {
  const reducedMotion = useReducedMotion();
  const { buttonProps, isArmed } = useConfirmAction({
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
  // Armed, the control reads as destructive the native way: red content on
  // an otherwise unchanged bezel. The colour lives on the armed content, which
  // crossfades, rather than on the button, whose colour would lerp into the
  // idle content as it returns.
  const contentClassName = cn(
    "absolute inset-0 flex items-center justify-center",
    isArmed && !isDisabled && "text-error",
  );

  if (variant === "text") {
    // Armed, the word gives way to the icon alone; the idle row reserves the
    // width so confirming never moves the layout.
    return (
      <Button
        {...pressProps}
        aria-label={isArmed ? armedLabel : undefined}
        className={cn("relative", buttonClassName)}
        isDisabled={isDisabled}
      >
        <span
          aria-hidden
          className="invisible inline-flex items-center gap-control"
        >
          {idleIcon}
          {idleLabel}
        </span>
        <AnimatePresence initial={false}>
          <motion.span
            key={isArmed ? "armed" : "idle"}
            {...SWAP_ANIMATION}
            className={cn(contentClassName, "gap-control")}
            transition={transition}
          >
            {isArmed ? armedIcon : idleIcon}
            {isArmed ? null : idleLabel}
          </motion.span>
        </AnimatePresence>
      </Button>
    );
  }

  return (
    <IconButton
      {...pressProps}
      aria-label={isArmed ? armedLabel : idleLabel}
      className={buttonClassName}
      isDisabled={isDisabled}
    >
      <span aria-hidden className="invisible flex items-center justify-center">
        {idleIcon}
      </span>
      <AnimatePresence initial={false}>
        <motion.span
          aria-hidden
          key={isArmed ? "armed" : "idle"}
          {...SWAP_ANIMATION}
          className={contentClassName}
          transition={transition}
        >
          {isArmed ? armedIcon : idleIcon}
        </motion.span>
      </AnimatePresence>
    </IconButton>
  );
}

export type { ConfirmActionButtonProps };
