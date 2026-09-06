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
import { IconButton, IconButtonProps } from "../../base/button/icon-button";

import { useConfirmAction } from "./use-confirm-action";

const SWAP_ANIMATION: MotionProps = {
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0 },
  initial: { opacity: 0, scale: 0 },
};

/**
 * The sizes both renderings offer. `Button` and `IconButton` name their sizes
 * alike, so the text variant is checked against the same union.
 */
type ConfirmActionButtonSize = NonNullable<IconButtonProps["size"]>;

type ConfirmActionButtonProps = {
  armedIcon: ReactNode;
  armedLabel: string;
  idleIcon: ReactNode;
  idleLabel: string;
  armedClassName?: string;
  className?: string;
  isDisabled?: boolean;
  onConfirm?: () => void;
  size?: ConfirmActionButtonSize;
  timeoutMs?: number;
  /** Icon-only, or an icon beside the label it is confirming. */
  variant?: "icon" | "text";
};

export function ConfirmActionButton({
  armedClassName,
  armedIcon,
  armedLabel,
  className,
  idleIcon,
  idleLabel,
  isDisabled,
  onConfirm,
  size = "default",
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
  const buttonClassName = cn(
    className,
    isArmed && !isDisabled && armedClassName,
    focusClassName,
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
        size={size}
      >
        <span
          aria-hidden
          className="gap-control-inset invisible inline-flex items-center"
        >
          {idleIcon}
          {idleLabel}
        </span>
        <AnimatePresence initial={false}>
          <motion.span
            key={isArmed ? "armed" : "idle"}
            {...SWAP_ANIMATION}
            className="gap-control-inset absolute inset-0 flex items-center justify-center"
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
      size={size}
    >
      <span aria-hidden className="invisible flex items-center justify-center">
        {idleIcon}
      </span>
      <AnimatePresence initial={false}>
        <motion.span
          aria-hidden
          key={isArmed ? "armed" : "idle"}
          {...SWAP_ANIMATION}
          className="absolute inset-0 flex items-center justify-center"
          transition={transition}
        >
          {isArmed ? armedIcon : idleIcon}
        </motion.span>
      </AnimatePresence>
    </IconButton>
  );
}

export type { ConfirmActionButtonProps };
