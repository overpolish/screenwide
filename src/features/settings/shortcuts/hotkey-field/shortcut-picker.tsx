// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ban, CircleAlert, TriangleAlert } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useEffect, useRef } from "react";
import {
  RadioButton,
  RadioField,
  RadioGroup,
  ToggleButton,
} from "react-aria-components";

import { platformArg, t } from "../../../../i18n/i18n";
import {
  type HotkeyModifier,
  hotkeyKey,
  hotkeyKeys,
  platformModifiers,
} from "../../../../lib/hotkey";
import { motionDurations } from "../../../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { fieldVariants } from "../../../../primitives/field/field-variants";
import {
  arrive,
  focusIn,
  glide,
  leave,
  reveal,
} from "../../../../primitives/island/island-motion";
import { keyLabel } from "../../../../primitives/keyboard/key-label";
import { Keyboard, Shortcut } from "../../../../primitives/keyboard/keyboard";
import { Text } from "../../../../primitives/text/text";

import type { ShortcutSuggestion } from "../shortcut-suggestions";
import type { PickStatus, ShortcutPick } from "./shortcut-pick";

// A key pressed alone, and the combinations it could make. The field shows
// the key with the modifiers beside it as toggles, so one can be added or
// taken away without pressing again; under it, the combinations offered,
// best first, the one in hand chosen. A combination that clashes says so
// on its chip and in the line under them, so it is never taken unseen.
// The chips ease in one after another, best first, as an island's parts
// are revealed (`IslandFlow`); for another key, the ones that stay glide
// to their new places and the rest blur away.

const still = { duration: 0 };

const keysText = (value: string) => hotkeyKeys(value).map(keyLabel).join(" + ");

const toggleClassName = cn(
  "inline-flex h-part-height min-w-part-height cursor-default items-center justify-center rounded-part px-inset-sm text-fg-tertiary",
  "data-[hovered]:bg-fill data-[selected]:bg-primary data-[selected]:text-on-primary data-[selected]:primary-raised",
  focusStyles,
  elementFocusVisible,
);

/** The field while a key is in hand: its modifiers as toggles, then the
 * key. */
export function PickerKeys({
  label,
  onModifiersChange,
  pick,
}: {
  label: string;
  onModifiersChange: (modifiers: HotkeyModifier[]) => void;
  pick: ShortcutPick;
}) {
  const key = hotkeyKey(pick.code);
  return (
    <div
      aria-label={t("hotkey-field-modifiers", { key: keyLabel(key), label })}
      className={cn(
        fieldVariants({ material: "raised" }).field(),
        "min-w-40 justify-center gap-0.5 px-inset-sm ring-3",
      )}
      dir="ltr"
      role="group"
    >
      {platformModifiers().map((modifier) => (
        <ToggleButton
          aria-label={keyLabel(hotkeyKey(modifier))}
          className={toggleClassName}
          data-modifier-toggle
          isSelected={pick.modifiers.includes(modifier)}
          key={modifier}
          onChange={(isSelected) => {
            onModifiersChange(
              isSelected
                ? [...pick.modifiers, modifier]
                : pick.modifiers.filter((other) => other !== modifier),
            );
          }}
        >
          <Keyboard className="text-current" variant="plain">
            {hotkeyKey(modifier)}
          </Keyboard>
        </ToggleButton>
      ))}
      <Keyboard className="ms-inline text-fg" variant="plain">
        {key}
      </Keyboard>
    </div>
  );
}

const chipName = ({ conflict, value }: ShortcutSuggestion) => {
  const keys = keysText(value);
  if (!conflict) return keys;
  if (conflict.kind === "taken")
    return t("hotkey-field-chip-taken", { action: conflict.label, keys });
  return conflict.by === "system"
    ? t("hotkey-field-chip-reserved-system", {
        keys,
        platform: platformArg(),
        use: conflict.label,
      })
    : t("hotkey-field-chip-reserved-apps", { keys, use: conflict.label });
};

/** The combinations offered, as chips to choose from, and the line saying
 * what the chosen one would do. Arrow keys move the choice; a click takes a
 * free chip at once. */
export function PickerChoices({
  current,
  onChoose,
  onTake,
  pick,
  status,
  suggestions,
}: {
  /** The combination in hand, if it is one of the chips. */
  current: string | null;
  onChoose: (suggestion: ShortcutSuggestion) => void;
  onTake: (value: string) => void;
  pick: ShortcutPick;
  status: PickStatus;
  suggestions: readonly ShortcutSuggestion[];
}) {
  const reduce = useReducedMotion() ?? false;
  const groupRef = useRef<HTMLDivElement>(null);
  // A new key brings the keyboard to its chips, on the one in hand.
  useEffect(() => {
    const group = groupRef.current;
    (
      group?.querySelector<HTMLInputElement>("input:checked") ??
      group?.querySelector<HTMLInputElement>("input:not(:disabled)")
    )?.focus();
  }, [pick.code]);
  return (
    <>
      <RadioGroup
        aria-label={t("hotkey-field-suggestions", {
          key: keyLabel(hotkeyKey(pick.code)),
        })}
        // Positioned, so a chip leaving is lifted out of the flow here.
        className="relative flex flex-wrap justify-end gap-inline"
        onChange={(value) => {
          const chosen = suggestions.find(
            (suggestion) => suggestion.value === value,
          );
          if (chosen) onChoose(chosen);
        }}
        orientation="horizontal"
        ref={groupRef}
        value={current}
      >
        <AnimatePresence mode="popLayout">
          {suggestions.map((suggestion, index) => {
            const isTaken = suggestion.conflict?.kind === "taken";
            const isReserved = suggestion.conflict?.kind === "reserved";
            return (
              <motion.div
                animate={reveal.animate}
                exit={{ ...focusIn.exit, transition: reduce ? still : leave }}
                initial={reveal.initial}
                key={suggestion.value}
                layout="position"
                transition={
                  reduce
                    ? still
                    : {
                        ...arrive,
                        delay: arrive.delay + index * motionDurations.feedback,
                        layout: glide,
                      }
                }
              >
                <RadioField
                  aria-label={chipName(suggestion)}
                  isDisabled={isTaken}
                  value={suggestion.value}
                >
                  <RadioButton
                    className={({ isSelected }) =>
                      cn(
                        "inline-flex h-control-height cursor-default items-center gap-inline rounded-control px-inset-sm text-fg",
                        isSelected
                          ? "bg-primary text-on-primary primary-raised"
                          : "control-raised data-[hovered]:control-raised-hover",
                        "data-[disabled]:opacity-40",
                        focusStyles,
                        elementFocusVisible,
                      )
                    }
                    // A click takes a free chip at once; the arrow keys only
                    // move the choice, which Return then takes. A reserved chip
                    // asks as much of a click as of the keys: the first click
                    // chooses it, so its warning is read, and a second takes it.
                    // The pointer lifts before the click chooses, so `current`
                    // is still the choice from before it.
                    onPointerUp={() => {
                      if (isTaken) return;
                      if (isReserved && current !== suggestion.value) return;
                      onTake(suggestion.value);
                    }}
                  >
                    <Shortcut aria-hidden>
                      {hotkeyKeys(suggestion.value).map((part) => (
                        <Keyboard
                          className="text-current"
                          key={part}
                          variant="plain"
                        >
                          {part}
                        </Keyboard>
                      ))}
                    </Shortcut>
                    {isTaken ? (
                      <Ban aria-hidden className="size-icon-sm" />
                    ) : suggestion.conflict ? (
                      <TriangleAlert
                        aria-hidden
                        className={cn(
                          "size-icon-sm",
                          current !== suggestion.value && "text-warning",
                        )}
                      />
                    ) : null}
                  </RadioButton>
                </RadioField>
              </motion.div>
            );
          })}
        </AnimatePresence>
      </RadioGroup>
      <Text
        aria-live="polite"
        className="flex max-w-72 items-start gap-inline text-end"
        variant="caption"
      >
        {status.tone === "warning" ? (
          <CircleAlert
            aria-hidden
            className="mt-px size-icon-sm shrink-0 text-warning"
          />
        ) : null}
        {status.text}
      </Text>
    </>
  );
}
