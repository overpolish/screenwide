// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleAlert, RotateCcw, X } from "lucide-react";
import { VisuallyHidden } from "react-aria";

import { t } from "../../../../i18n/i18n";
import {
  type HotkeyCaptureMode,
  hotkeyKeys,
  hotkeysEqual,
} from "../../../../lib/hotkey";
import { cn } from "../../../../lib/styling";
import { IconButton } from "../../../../primitives/button/icon-button";
import { SettingsReveal } from "../../settings-reveal";

import { FieldSwap } from "./field-swap";
import { HotkeyTrigger } from "./hotkey-trigger";
import { PickerChoices, PickerKeys } from "./shortcut-picker";
import { useHotkeyCapture } from "./use-hotkey-capture";

import type { ShortcutConflict } from "../shortcut-conflicts";

// A shortcut, set by pressing it. One that works anywhere needs no chord
// held down: a key pressed alone opens its combinations to pick from, best
// first, with the modifiers as toggles beside the key. A whole chord
// pressed is taken at once when nothing else has it.

export type HotkeyFieldProps = {
  "aria-label": string;
  onChange: (value: string | null) => void;
  value: string | null;
  "aria-describedby"?: string;
  /** The row title naming the field; `aria-label` still names its buttons. */
  "aria-labelledby"?: string;
  captureMode?: HotkeyCaptureMode;
  className?: string;
  /** What a combination would clash with, so the field marks it and never
   * takes it unseen. */
  conflictOf?: (value: string) => ShortcutConflict | null;
  defaultValue?: string | null;
  isClearable?: boolean;
  isDisabled?: boolean;
  /** Capture waits for the host to suspend shortcuts; cleanup reports false. */
  onCaptureChange?: (capturing: boolean) => void | Promise<void>;
};

export function HotkeyField({
  "aria-describedby": describedBy,
  "aria-label": label,
  "aria-labelledby": labelledBy,
  captureMode = "shortcut",
  className,
  conflictOf,
  defaultValue,
  isClearable = true,
  isDisabled = false,
  onCaptureChange,
  onChange,
  value,
}: HotkeyFieldProps) {
  const capture = useHotkeyCapture({
    captureMode,
    conflictOf,
    isClearable,
    isDisabled,
    onCaptureChange,
    onChange,
  });
  const { buttonRef, capturing, interactionFocus, pick, status } = capture;
  const single = captureMode === "single-control";
  const local = captureMode === "local-shortcut";
  const instructions = single
    ? t("hotkey-field-instructions-single")
    : local
      ? t("hotkey-field-instructions-local")
      : isClearable
        ? t("hotkey-field-instructions-clearable")
        : t("hotkey-field-instructions");

  return (
    <div
      // The parts under the row bring their own gap, so it opens with them.
      className={cn("inline-flex flex-col items-end", className)}
      {...capture.rootProps}
    >
      <div className="inline-flex items-center gap-inline">
        {/* Before the field, so the field never moves when it appears. */}
        {defaultValue !== undefined && !hotkeysEqual(value, defaultValue) ? (
          <IconButton
            aria-label={t("hotkey-field-reset", { label })}
            isDisabled={isDisabled}
            onPress={() => {
              capture.stop();
              onChange(defaultValue);
              capture.setFeedback(t("hotkey-field-reset-done"));
              buttonRef.current?.focus();
            }}
            variant="ghost"
          >
            <RotateCcw />
          </IconButton>
        ) : null}
        <FieldSwap swapKey={pick ? "pick" : "trigger"}>
          {pick ? (
            <PickerKeys
              label={label}
              onModifiersChange={(modifiers) => {
                capture.setPick({ ...pick, modifiers });
              }}
              pick={pick}
            />
          ) : (
            <HotkeyTrigger
              aria-describedby={describedBy}
              aria-labelledby={labelledBy}
              capturing={capturing}
              className={interactionFocus.className}
              instructions={instructions}
              isDisabled={isDisabled}
              keys={hotkeyKeys(value)}
              label={label}
              mouseProgress={capture.mouseProgress}
              onPress={(event) => {
                interactionFocus.onPress(event);
                buttonRef.current?.focus();
                capture.start();
                capture.setFeedback(
                  capturing ? t("hotkey-field-cancelled") : instructions,
                );
              }}
              prompt={
                single
                  ? t("hotkey-field-press-key-or-button")
                  : t("hotkey-field-press-shortcut")
              }
              ready={capture.ready}
              ref={buttonRef}
            />
          )}
        </FieldSwap>
        <IconButton
          aria-label={t("hotkey-field-clear", { label })}
          // The slot is kept even with nothing to clear, or a key that can
          // never be cleared, so every field in a list of shortcuts lines up;
          // then it is neither seen nor reached.
          className={!isClearable || value === null ? "invisible" : undefined}
          isDisabled={isDisabled || !isClearable}
          onPress={() => {
            capture.commit(null);
            buttonRef.current?.focus();
          }}
          variant="ghost"
        >
          <X />
        </IconButton>
      </div>
      <SettingsReveal className="flex flex-col items-end gap-inline pt-inline">
        {pick && status ? (
          <PickerChoices
            current={capture.current}
            onChoose={(suggestion) => {
              capture.setPick({ ...pick, modifiers: suggestion.modifiers });
            }}
            onTake={capture.take}
            pick={pick}
            status={status}
            suggestions={capture.suggestions}
          />
        ) : null}
      </SettingsReveal>
      <SettingsReveal className="pt-inline">
        {/* The glyph carries the error colour; the words stay readable. */}
        {capture.captureError ? (
          <p
            className="flex max-w-64 items-start gap-inline text-end text-caption text-fg-secondary"
            role="alert"
          >
            <CircleAlert className="mt-px size-icon-sm shrink-0 text-error" />
            {capture.captureError}
          </p>
        ) : null}
      </SettingsReveal>
      <VisuallyHidden>
        <span aria-live="polite" role="status">
          {capture.feedback}
        </span>
      </VisuallyHidden>
    </div>
  );
}
