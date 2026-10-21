// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleAlert, RotateCcw, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { VisuallyHidden } from "react-aria";
import { Button } from "react-aria-components";

import { t } from "../../../i18n/i18n";
import {
  type HotkeyCaptureMode,
  hotkeyFromEvent,
  hotkeyKeys,
  hotkeysEqual,
} from "../../../lib/hotkey";
import { cn, elementFocusVisible } from "../../../lib/styling";
import { useInteractionFocus } from "../../../lib/use-interaction-focus";
import { IconButton } from "../../../primitives/button/icon-button";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { fieldVariants } from "../../../primitives/field/field-variants";
import { keyLabel } from "../../../primitives/keyboard/key-label";
import { Keyboard, Shortcut } from "../../../primitives/keyboard/keyboard";

import { useMouseControlCapture } from "./use-mouse-control-capture";

// The recorder reads as a field pressed to set, the raised field a select
// is drawn in: the shortcut as keycaps, or a placeholder while there is
// none. While it records, the prompt stands in for the keys and the focus
// ring stays on, as the field being typed into keeps it.
const triggerClassName = cn(
  fieldVariants({ material: "raised" }).field(),
  // Centred: with no chevron to balance it, a value at the leading edge
  // would leave the rest of the field empty.
  "w-40 cursor-default justify-center gap-inline px-inset text-body",
  "data-[hovered]:control-raised-hover data-[pressed]:control-raised-pressed data-[disabled]:opacity-40",
  elementFocusVisible,
);

export type HotkeyFieldProps = {
  "aria-label": string;
  onChange: (value: string | null) => void;
  value: string | null;
  "aria-describedby"?: string;
  captureMode?: HotkeyCaptureMode;
  className?: string;
  defaultValue?: string | null;
  isClearable?: boolean;
  isDisabled?: boolean;
  /** Capture waits for the host to suspend shortcuts; cleanup reports false. */
  onCaptureChange?: (capturing: boolean) => void | Promise<void>;
};

export function HotkeyField({
  "aria-describedby": describedBy,
  "aria-label": label,
  captureMode = "shortcut",
  className,
  defaultValue,
  isClearable = true,
  isDisabled = false,
  onCaptureChange,
  onChange,
  value,
}: HotkeyFieldProps) {
  const [listening, setListening] = useState(false);
  const [feedback, setFeedback] = useState("");
  const [captureError, setCaptureError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);
  const interactionFocus = useInteractionFocus();
  const buttonRef = useRef<HTMLButtonElement>(null);
  const capturing = listening && !isDisabled;
  const single = captureMode === "single-control";
  const local = captureMode === "local-shortcut";
  const instructions = single
    ? t("hotkey-field-instructions-single")
    : local
      ? t("hotkey-field-instructions-local")
      : isClearable
        ? t("hotkey-field-instructions-clearable")
        : t("hotkey-field-instructions");
  // Reset during render so re-enabling cannot resume an old capture session.
  if (isDisabled && listening) setListening(false);
  const keys = hotkeyKeys(value);
  const commit = (next: string) => {
    setListening(false);
    setReady(false);
    onChange(next);
    setFeedback(t("hotkey-field-updated"));
  };
  const mouseProgress = useMouseControlCapture(
    capturing && ready && single,
    commit,
    () => {
      setCaptureError(t("hotkey-field-hold-error"));
    },
  );

  useEffect(() => {
    if (!capturing) return;
    let disposed = false;
    const started = Promise.resolve().then(() => onCaptureChange?.(true));
    void started
      .then(() => {
        if (!disposed) setReady(true);
      })
      .catch((reason: unknown) => {
        if (!disposed) {
          setListening(false);
          setCaptureError(
            t("hotkey-field-capture-failed", { error: String(reason) }),
          );
        }
      });
    const cancel = () => {
      setListening(false);
      setReady(false);
      setFeedback(t("hotkey-field-cancelled"));
    };
    window.addEventListener("blur", cancel);
    return () => {
      disposed = true;
      window.removeEventListener("blur", cancel);
      // Always release, even when cancellation races asynchronous preparation.
      void started
        .catch(() => undefined)
        .then(() => onCaptureChange?.(false))
        .catch((reason: unknown) => {
          console.error("Could not restore shortcuts after capture", reason);
        });
    };
  }, [capturing, onCaptureChange]);

  return (
    <div
      className={cn("inline-flex flex-col items-end gap-inline", className)}
      onKeyDownCapture={(event) => {
        // Captured keys are data entry, not a switch to keyboard navigation.
        // The next ordinary key restores normal focus-visible behaviour.
        if (!capturing) {
          interactionFocus.onKeyDown(false);
          return;
        }
        // Local shortcuts capture Tab; Escape always cancels capture.
        if (
          !local &&
          event.key === "Tab" &&
          !event.altKey &&
          !event.ctrlKey &&
          !event.metaKey
        ) {
          interactionFocus.onKeyDown(false);
          setListening(false);
          setReady(false);
          return;
        }
        interactionFocus.onKeyDown(true);
        event.preventDefault();
        event.stopPropagation();
        if (event.nativeEvent.isComposing || event.repeat) return;
        if (event.key === "Escape") {
          setListening(false);
          setReady(false);
          setFeedback(t("hotkey-field-cancelled"));
          return;
        }
        if (!ready) return;
        if (
          !single &&
          !local &&
          isClearable &&
          (event.key === "Backspace" || event.key === "Delete")
        ) {
          setListening(false);
          setReady(false);
          onChange(null);
          setFeedback(t("hotkey-field-cleared"));
          return;
        }
        const next = hotkeyFromEvent(event.nativeEvent, captureMode);
        if (!next) return;
        commit(next);
      }}
    >
      <div className="inline-flex items-center gap-inline">
        {/* Before the field, so the field never moves when it appears. */}
        {defaultValue !== undefined && !hotkeysEqual(value, defaultValue) ? (
          <IconButton
            aria-label={t("hotkey-field-reset", { label })}
            isDisabled={isDisabled}
            onPress={() => {
              setListening(false);
              setReady(false);
              onChange(defaultValue);
              setFeedback(t("hotkey-field-reset-done"));
              buttonRef.current?.focus();
            }}
            variant="ghost"
          >
            <RotateCcw />
          </IconButton>
        ) : null}
        <Button
          aria-describedby={describedBy}
          aria-description={instructions}
          aria-label={t("hotkey-field-value", {
            keys: keys.map(keyLabel).join(" + ") || t("hotkey-field-not-set"),
            label,
          })}
          aria-pressed={capturing}
          className={cn(
            triggerClassName,
            capturing && "ring-3",
            interactionFocus.className,
          )}
          isDisabled={isDisabled}
          onBlur={() => {
            interactionFocus.onBlur();
            setListening(false);
            setReady(false);
            if (capturing) setFeedback(t("hotkey-field-cancelled"));
          }}
          onPress={(event) => {
            interactionFocus.onPress(event);
            buttonRef.current?.focus();
            setReady(false);
            setCaptureError(null);
            setListening(!capturing);
            setFeedback(capturing ? t("hotkey-field-cancelled") : instructions);
          }}
          ref={buttonRef}
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
              ) : single ? (
                t("hotkey-field-press-key-or-button")
              ) : (
                t("hotkey-field-press-shortcut")
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
        </Button>
        <IconButton
          aria-label={t("hotkey-field-clear", { label })}
          // The slot is kept even with nothing to clear, or a key that can
          // never be cleared, so every field in a list of shortcuts lines up;
          // then it is neither seen nor reached.
          className={!isClearable || value === null ? "invisible" : undefined}
          isDisabled={isDisabled || !isClearable}
          onPress={() => {
            setListening(false);
            setReady(false);
            onChange(null);
            setFeedback(t("hotkey-field-cleared"));
            buttonRef.current?.focus();
          }}
          variant="ghost"
        >
          <X />
        </IconButton>
      </div>
      {/* The glyph carries the error colour; the words stay readable. */}
      {captureError ? (
        <p
          className="flex max-w-64 items-start gap-1 text-end text-caption text-fg-secondary"
          role="alert"
        >
          <CircleAlert className="mt-px size-icon-sm shrink-0 text-error" />
          {captureError}
        </p>
      ) : null}
      <VisuallyHidden>
        <span aria-live="polite" role="status">
          {feedback}
        </span>
      </VisuallyHidden>
    </div>
  );
}
