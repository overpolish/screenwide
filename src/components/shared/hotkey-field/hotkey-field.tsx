// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { VisuallyHidden } from "react-aria";

import { cn } from "../../../lib/styling";
import { useInteractionFocus } from "../../../lib/use-interaction-focus";
import { Button } from "../../base/button/button";
import { IconButton } from "../../base/button/icon-button";
import { CircularProgress } from "../../base/circular-progress/circular-progress";
import { Keyboard, Shortcut } from "../../base/keyboard/keyboard";
import { Text } from "../../base/text/text";

import { type HotkeyCaptureMode, hotkeyFromEvent, hotkeyKeys } from "./hotkey";
import { useMouseControlCapture } from "./use-mouse-control-capture";

export type HotkeyFieldProps = {
  "aria-label": string;
  onChange: (value: string | null) => void;
  value: string | null;
  "aria-describedby"?: string;
  captureMode?: HotkeyCaptureMode;
  className?: string;
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
  const instructions = single
    ? "Press a key or hold and release an auxiliary mouse button. Escape cancels; Tab leaves."
    : `Use a modifier and a key. Escape cancels; Tab leaves.${isClearable ? " Delete clears." : ""}`;
  // Reset during render so re-enabling cannot resume an old capture session.
  if (isDisabled && listening) setListening(false);
  const keys = hotkeyKeys(
    value,
    typeof navigator !== "undefined" && navigator.userAgent.includes("Mac"),
  );
  const commit = (next: string) => {
    setListening(false);
    setReady(false);
    onChange(next);
    setFeedback("Shortcut updated.");
  };
  const mouseProgress = useMouseControlCapture(
    capturing && ready && single,
    commit,
    () => {
      setCaptureError("Hold the button for half a second, then release.");
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
          setCaptureError(`Could not start capture: ${String(reason)}`);
        }
      });
    const cancel = () => {
      setListening(false);
      setReady(false);
      setFeedback("Shortcut capture cancelled.");
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
        // Tab remains available to leave the control without trapping focus.
        if (
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
          setFeedback("Shortcut capture cancelled.");
          return;
        }
        if (!ready) return;
        if (
          !single &&
          isClearable &&
          (event.key === "Backspace" || event.key === "Delete")
        ) {
          setListening(false);
          setReady(false);
          onChange(null);
          setFeedback("Shortcut cleared.");
          return;
        }
        const next = hotkeyFromEvent(event.nativeEvent, captureMode);
        if (!next) return;
        commit(next);
      }}
    >
      <div className="gap-control-inset inline-flex items-center">
        <Button
          aria-describedby={describedBy}
          aria-description={instructions}
          aria-label={`${label}: ${keys.join(" + ") || "Not set"}`}
          aria-pressed={capturing}
          className={interactionFocus.className}
          isDisabled={isDisabled}
          onBlur={() => {
            interactionFocus.onBlur();
            setListening(false);
            setReady(false);
            if (capturing) setFeedback("Shortcut capture cancelled.");
          }}
          onPress={(event) => {
            interactionFocus.onPress(event);
            buttonRef.current?.focus();
            setReady(false);
            setCaptureError(null);
            setListening(!capturing);
            setFeedback(
              capturing ? "Shortcut capture cancelled." : instructions,
            );
          }}
          ref={buttonRef}
        >
          {capturing ? (
            !ready ? (
              "Preparing"
            ) : mouseProgress !== null ? (
              <>
                <CircularProgress
                  aria-label="Testing button hold"
                  size="compact"
                  value={mouseProgress}
                />
                Keep holding
              </>
            ) : single ? (
              "Press key or hold button"
            ) : (
              "Press shortcut"
            )
          ) : keys.length ? (
            <Shortcut>
              {keys.map((key, index) => (
                <Keyboard key={`${key}-${index.toString()}`}>{key}</Keyboard>
              ))}
            </Shortcut>
          ) : (
            "Set shortcut"
          )}
        </Button>
        {isClearable ? (
          <IconButton
            aria-label={`Clear ${label}`}
            isDisabled={isDisabled || value === null}
            onPress={() => {
              setListening(false);
              setReady(false);
              onChange(null);
              setFeedback("Shortcut cleared.");
              buttonRef.current?.focus();
            }}
          >
            <X />
          </IconButton>
        ) : null}
      </div>
      {captureError ? (
        <Text
          className="max-w-64 text-right text-error"
          role="alert"
          variant="help"
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
