// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Ref, useId } from "react";
import { Button, type PressEvent } from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { IslandSwap } from "../../../../primitives/island/island-swap";
import { keyLabel } from "../../../../primitives/keyboard/key-label";
import { Keyboard, Shortcut } from "../../../../primitives/keyboard/keyboard";

import { hotkeyTriggerClassName } from "./hotkey-trigger-style";

// The shortcut as keycaps, or a placeholder while there is none. While it
// records, the prompt stands in for the keys. The focus ring follows how it
// was reached, as on any button: a press by pointer shows none, and the
// prompt alone says it is listening. What it shows swaps as an island's
// parts do (`IslandSwap`): the keys blur away and the prompt clears in where
// they were.

export function HotkeyTrigger({
  "aria-describedby": describedBy,
  "aria-labelledby": labelledBy,
  capturing,
  className,
  instructions,
  isDisabled,
  keys,
  label,
  mouseProgress,
  onPress,
  prompt,
  ready,
  ref,
}: {
  "aria-describedby": string | undefined;
  /** The row's title, when one names the field: the name then reads the
   * title and the keys, and the window search finds the field by it. */
  "aria-labelledby": string | undefined;
  capturing: boolean;
  /** How it is used, read out with its name. */
  instructions: string;
  isDisabled: boolean;
  keys: string[];
  label: string;
  /** How long a mouse button has been held, from 0 to 1; null when none. */
  mouseProgress: number | null;
  onPress: (event: PressEvent) => void;
  /** What to press, shown while it records. */
  prompt: string;
  ready: boolean;
  ref: Ref<HTMLButtonElement>;
  className?: string;
}) {
  const id = useId();
  const shown = keys.map(keyLabel).join(" + ") || t("hotkey-field-not-set");
  // Each thing it can show swaps with the others; the hold's progress
  // changes within its own.
  const showing = capturing
    ? !ready
      ? "preparing"
      : mouseProgress !== null
        ? "holding"
        : "prompt"
    : `keys:${keys.join("+")}`;
  return (
    <Button
      aria-describedby={describedBy}
      aria-description={instructions}
      aria-label={
        labelledBy ? shown : t("hotkey-field-value", { keys: shown, label })
      }
      // Naming itself last reads its own label, the keys, after the title.
      aria-labelledby={labelledBy ? `${labelledBy} ${id}` : undefined}
      aria-pressed={capturing}
      // Positioned, so what it showed is lifted out of the flow as it goes.
      className={cn(hotkeyTriggerClassName, "relative", className)}
      id={id}
      isDisabled={isDisabled}
      onPress={onPress}
      ref={ref}
    >
      <IslandSwap
        className="flex min-w-0 items-center"
        initial={false}
        swapKey={showing}
      >
        {capturing ? (
          <span className="flex min-w-0 items-center gap-inline truncate text-fg-secondary">
            {!ready ? (
              t("hotkey-field-preparing")
            ) : mouseProgress !== null ? (
              <>
                <CircularProgress
                  aria-label={t("hotkey-field-testing-hold")}
                  size="small"
                  value={mouseProgress}
                />
                {t("hotkey-field-keep-holding")}
              </>
            ) : (
              prompt
            )}
          </span>
        ) : keys.length ? (
          <Shortcut>
            {keys.map((key, index) => (
              <Keyboard key={`${key}-${index.toString()}`}>{key}</Keyboard>
            ))}
          </Shortcut>
        ) : (
          <span className="truncate text-fg-tertiary">
            {t("hotkey-field-set")}
          </span>
        )}
      </IslandSwap>
    </Button>
  );
}
