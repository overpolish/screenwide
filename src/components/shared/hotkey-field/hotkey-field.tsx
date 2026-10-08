// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { VisuallyHidden } from "react-aria";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { useInteractionFocus } from "../../../lib/use-interaction-focus";
import { Button } from "../../base/button/button";
import { IconButton } from "../../base/button/icon-button";
import { CircularProgress } from "../../base/circular-progress/circular-progress";
import { keyLabel } from "../../base/keyboard/key-label";
import { Keyboard, Shortcut } from "../../base/keyboard/keyboard";
import { Text } from "../../base/text/text";

import {
  type HotkeyCaptureMode,
  hotkeyFromEvent,
  hotkeyKeys,
  hotkeysEqual,
} from "./hotkey";
import { useMouseControlCapture } from "./use-mouse-control-capture";

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
  const isMac =
    typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
  const instructions = single
    ? t("hotkey-field-instructions-single")
    : local
      ? t("hotkey-field-instructions-local")
      : isClearable
        ? t("hotkey-field-instructions-clearable")
        : t("hotkey-field-instructions");
  // Reset during render so re-enabling cannot resume an old capture session.
  if (isDisabled && listening) setListening(false);
  const keys = hotkeyKeys(value, isMac);
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
      className={cn("gap-control inline-flex flex-col items-end", className)}
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
      <div className="gap-control inline-flex items-center">
        <Button
          aria-describedby={describedBy}
          aria-description={instructions}
          aria-label={t("hotkey-field-value", {
            keys: keys.map(keyLabel).join(" + ") || t("hotkey-field-not-set"),
            label,
          })}
          aria-pressed={capturing}
          className={interactionFocus.className}
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
            !ready ? (
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
            )
          ) : keys.length ? (
            <Shortcut>
              {keys.map((key, index) => (
                <Keyboard key={`${key}-${index.toString()}`} variant="plain">
                  {key}
                </Keyboard>
              ))}
            </Shortcut>
          ) : (
            t("hotkey-field-set")
          )}
        </Button>
        {isClearable ? (
          <IconButton
            aria-label={t("hotkey-field-clear", { label })}
            isDisabled={isDisabled || value === null}
            onPress={() => {
              setListening(false);
              setReady(false);
              onChange(null);
              setFeedback(t("hotkey-field-cleared"));
              buttonRef.current?.focus();
            }}
          >
            <X />
          </IconButton>
        ) : null}
        {defaultValue !== undefined &&
        !hotkeysEqual(value, defaultValue, isMac) ? (
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
          >
            <RotateCcw />
          </IconButton>
        ) : null}
      </div>
      {captureError ? (
        <Text
          className="max-w-64 text-end text-error"
          role="alert"
          variant="subheadline"
        >
          {captureError}
        </Text>
      ) : null}
      <VisuallyHidden>
        <span aria-live="polite" role="status">
          {feedback}
        </span>
      </VisuallyHidden>
    </div>
  );
}
