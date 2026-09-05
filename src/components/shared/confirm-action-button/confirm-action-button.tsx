// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnimatePresence,
  motion,
  MotionProps,
  useReducedMotion,
} from "motion/react";
import { ReactNode, useEffect, useRef, useState } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { useInteractionFocus } from "../../../lib/use-interaction-focus";
import { IconButton, IconButtonProps } from "../../base/button/icon-button";

const DEFAULT_CONFIRM_TIMEOUT_MS = 2_000;
const ICON_SWAP_ANIMATION: MotionProps = {
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0 },
  initial: { opacity: 0, scale: 0 },
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
  timeoutMs = DEFAULT_CONFIRM_TIMEOUT_MS,
}: {
  armedIcon: ReactNode;
  armedLabel: string;
  idleIcon: ReactNode;
  idleLabel: string;
  armedClassName?: string;
  className?: string;
  isDisabled?: boolean;
  onConfirm?: () => void;
  size?: IconButtonProps["size"];
  timeoutMs?: number;
}) {
  const [isArmed, setIsArmed] = useState(false);
  const disarmRef = useRef<number | undefined>(undefined);
  const reducedMotion = useReducedMotion();
  const interactionFocus = useInteractionFocus();
  const buttonRef = useRef<HTMLButtonElement>(null);
  // A disabled confirmation must not remain armed when re-enabled.

  if (isDisabled && isArmed) setIsArmed(false);
  const disarm = () => {
    window.clearTimeout(disarmRef.current);
    setIsArmed(false);
  };

  useEffect(
    () => () => {
      window.clearTimeout(disarmRef.current);
    },
    [],
  );

  return (
    <IconButton
      aria-label={isArmed ? armedLabel : idleLabel}
      className={cn(
        className,
        isArmed && !isDisabled && armedClassName,
        interactionFocus.className,
      )}
      isDisabled={isDisabled}
      onBlur={() => {
        interactionFocus.onBlur();
        disarm();
      }}
      onKeyDown={(event) => {
        const cancelling = event.key === "Escape" && isArmed;
        interactionFocus.onKeyDown(cancelling);
        if (!cancelling) return;
        event.preventDefault();
        event.stopPropagation();
        disarm();
      }}
      onPress={(event) => {
        interactionFocus.onPress(event);
        buttonRef.current?.focus();
        window.clearTimeout(disarmRef.current);

        if (isArmed) {
          setIsArmed(false);
          onConfirm?.();
          return;
        }

        setIsArmed(true);
        disarmRef.current = window.setTimeout(() => {
          setIsArmed(false);
        }, timeoutMs);
      }}
      ref={buttonRef}
      size={size}
    >
      <span aria-hidden className="invisible flex items-center justify-center">
        {idleIcon}
      </span>
      <AnimatePresence initial={false}>
        <motion.span
          aria-hidden
          key={isArmed ? "armed" : "idle"}
          {...ICON_SWAP_ANIMATION}
          className="absolute inset-0 flex items-center justify-center"
          transition={{
            duration: reducedMotion ? 0 : motionDurations.state,
            ease: motionEasings.out,
          }}
        >
          {isArmed ? armedIcon : idleIcon}
        </motion.span>
      </AnimatePresence>
    </IconButton>
  );
}
