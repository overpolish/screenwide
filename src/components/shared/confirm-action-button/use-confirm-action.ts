// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { KeyboardEvent, Ref, useEffect, useRef, useState } from "react";

import { useInteractionFocus } from "../../../lib/use-interaction-focus";

import type { PressEvent } from "react-aria";

const DEFAULT_CONFIRM_TIMEOUT_MS = 2_000;

export type ConfirmActionTriggerProps = {
  className: string | undefined;
  onBlur: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => void;
  onPress: (event: PressEvent) => void;
  ref: Ref<HTMLButtonElement>;
};

/**
 * The arm-then-confirm state machine, shared by every rendering of it.
 *
 * The first press arms the action and starts a short window in which a second
 * press confirms it; leaving the button, pressing Escape, the window expiring,
 * or the action becoming unavailable all disarm it again.
 */
export function useConfirmAction({
  isDisabled,
  onConfirm,
  timeoutMs = DEFAULT_CONFIRM_TIMEOUT_MS,
}: {
  isDisabled?: boolean;
  onConfirm?: () => void;
  timeoutMs?: number;
}): { buttonProps: ConfirmActionTriggerProps; isArmed: boolean } {
  const [isArmed, setIsArmed] = useState(false);
  const disarmRef = useRef<number | undefined>(undefined);
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

  return {
    buttonProps: {
      className: interactionFocus.className,
      onBlur: () => {
        interactionFocus.onBlur();
        disarm();
      },
      onKeyDown: (event) => {
        const cancelling = event.key === "Escape" && isArmed;
        interactionFocus.onKeyDown(cancelling);
        if (!cancelling) return;
        event.preventDefault();
        event.stopPropagation();
        disarm();
      },
      onPress: (event) => {
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
      },
      ref: buttonRef,
    },
    isArmed,
  };
}
