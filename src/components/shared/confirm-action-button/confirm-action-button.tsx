// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, MotionProps } from "motion/react";
import { ReactNode, useEffect, useRef, useState } from "react";

import { IconButton, IconButtonProps } from "../../base/button/icon-button";

const DEFAULT_CONFIRM_TIMEOUT_MS = 2_000;
const ICON_SWAP_ANIMATION: MotionProps = {
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0 },
  initial: { opacity: 0, scale: 0 },
};

export function ConfirmActionButton({
  armedIcon,
  armedLabel,
  className,
  idleIcon,
  idleLabel,
  isDisabled,
  onConfirm,
  size,
  timeoutMs = DEFAULT_CONFIRM_TIMEOUT_MS,
}: {
  armedIcon: ReactNode;
  armedLabel: string;
  idleIcon: ReactNode;
  idleLabel: string;
  className?: string;
  isDisabled?: boolean;
  onConfirm?: () => void;
  size?: IconButtonProps["size"];
  timeoutMs?: number;
}) {
  const [isArmed, setIsArmed] = useState(false);
  const disarmRef = useRef<number | undefined>(undefined);

  useEffect(
    () => () => {
      window.clearTimeout(disarmRef.current);
    },
    [],
  );

  return (
    <IconButton
      aria-label={isArmed ? armedLabel : idleLabel}
      className={className}
      isDisabled={isDisabled}
      onPress={() => {
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
      size={size}
    >
      <span className="invisible flex items-center justify-center">
        {idleIcon}
      </span>
      <AnimatePresence initial={false}>
        <motion.span
          key={isArmed ? "armed" : "idle"}
          {...ICON_SWAP_ANIMATION}
          className="absolute inset-0 flex items-center justify-center"
        >
          {isArmed ? armedIcon : idleIcon}
        </motion.span>
      </AnimatePresence>
    </IconButton>
  );
}
