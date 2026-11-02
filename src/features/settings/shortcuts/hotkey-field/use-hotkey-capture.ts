// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type FocusEvent,
  type KeyboardEvent,
  type MouseEvent,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import { t } from "../../../../i18n/i18n";
import {
  type HotkeyCaptureMode,
  hotkeyFromEvent,
  type HotkeyModifier,
  hotkeyKey,
  hotkeyKeys,
  hotkeyValue,
  isGlobalChord,
  keyCodeFromEvent,
  parseHotkey,
} from "../../../../lib/hotkey";
import { isWindows } from "../../../../lib/platform";
import { useInteractionFocus } from "../../../../lib/use-interaction-focus";
import { keyLabel } from "../../../../primitives/keyboard/key-label";
import { bestSuggestion, suggestShortcuts } from "../shortcut-suggestions";

import { pickStatus, type ShortcutPick } from "./shortcut-pick";
import { useCaptureSession } from "./use-capture-session";
import { useMouseControlCapture } from "./use-mouse-control-capture";

import type { ShortcutConflict } from "../shortcut-conflicts";

// What a shortcut field does with the keys while it records. A shortcut
// that works anywhere takes a whole chord at once when it is free; a key
// pressed alone, or a chord that clashes, puts the key in hand and offers
// its combinations. A local shortcut takes a key alone, and a held control
// any key or mouse button; neither takes a key another shortcut has.

// Keys that move between the picker's parts rather than make a shortcut.
const NAVIGATION = [
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "End",
  "Home",
];

const hasModifier = (event: KeyboardEvent) =>
  event.altKey || event.ctrlKey || event.metaKey || event.shiftKey;

export function useHotkeyCapture({
  captureMode,
  conflictOf,
  isClearable,
  isDisabled,
  onCaptureChange,
  onChange,
}: {
  captureMode: HotkeyCaptureMode;
  isClearable: boolean;
  isDisabled: boolean;
  onChange: (value: string | null) => void;
  conflictOf?: (value: string) => ShortcutConflict | null;
  onCaptureChange?: (capturing: boolean) => void | Promise<void>;
}) {
  const [listening, setListening] = useState(false);
  const [ready, setReady] = useState(false);
  const [pick, setPick] = useState<ShortcutPick | null>(null);
  const [feedback, setFeedback] = useState("");
  const [captureError, setCaptureError] = useState<string | null>(null);
  const interactionFocus = useInteractionFocus();
  const rootRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const refocusRef = useRef(false);
  const capturing = listening && !isDisabled;
  const single = captureMode === "single-control";
  const local = captureMode === "local-shortcut";
  const isMac = !isWindows();
  // Reset during render so re-enabling cannot resume an old capture session.
  if (isDisabled && listening) setListening(false);
  if (!capturing && pick) setPick(null);
  const conflictFor = (candidate: string) => conflictOf?.(candidate) ?? null;

  // The field comes back once a pick ends or a shortcut is taken, so the
  // keyboard returns to it. Saving disables the field, which drops its
  // focus, so this waits for it to be enabled again; focus the person has
  // since moved elsewhere is left where it is.
  useLayoutEffect(() => {
    const button = buttonRef.current;
    if (!refocusRef.current || pick || !button || button.disabled) return;
    refocusRef.current = false;
    const focused = document.activeElement;
    if (
      !focused ||
      focused === document.body ||
      rootRef.current?.contains(focused)
    )
      button.focus();
  });

  const stop = () => {
    refocusRef.current = pick !== null;
    setListening(false);
    setReady(false);
    setPick(null);
  };
  const cancel = () => {
    stop();
    setFeedback(t("hotkey-field-cancelled"));
  };
  const commit = (next: string | null) => {
    stop();
    refocusRef.current = true;
    setCaptureError(null);
    onChange(next);
    setFeedback(
      next === null ? t("hotkey-field-cleared") : t("hotkey-field-updated"),
    );
  };
  const mouseProgress = useMouseControlCapture(
    capturing && ready && single,
    commit,
    () => {
      setCaptureError(t("hotkey-field-hold-error"));
    },
  );
  useCaptureSession({
    capturing,
    onCancel: cancel,
    onCaptureChange,
    onFailed: (message) => {
      setListening(false);
      setCaptureError(message);
    },
    onReady: () => {
      setReady(true);
    },
  });

  const offers = (code: string, include: HotkeyModifier[][]) =>
    suggestShortcuts(code, { conflictOf: conflictFor, include, isMac });
  const suggestions = pick
    ? offers(pick.code, [...pick.held, pick.modifiers])
    : [];
  const current = pick ? hotkeyValue(pick.code, pick.modifiers) : null;
  const status =
    pick && current ? pickStatus(pick, conflictFor(current)) : null;

  // A key in hand offers its best free combination first; a chord held
  // that clashed stays among the offers, marked, for choosing knowingly.
  const startPick = (code: string, held?: HotkeyModifier[]) => {
    const best = bestSuggestion(offers(code, held ? [held] : []));
    setPick({
      code,
      held: held ? [held] : [],
      modifiers: best?.modifiers ?? held ?? [],
    });
    setFeedback(
      t("hotkey-field-pick-start", { key: keyLabel(hotkeyKey(code)) }),
    );
  };
  // Never a combination another shortcut has, nor one Shift alone makes.
  const take = (candidate: string) => {
    const modifiers = parseHotkey(candidate, isMac)?.modifiers ?? [];
    if (isGlobalChord(modifiers) && conflictFor(candidate)?.kind !== "taken")
      commit(candidate);
    else if (status) setFeedback(status.text);
  };

  const onKeyDownCapture = (event: KeyboardEvent<HTMLDivElement>) => {
    // Captured keys are data entry, not a switch to keyboard navigation.
    // The next ordinary key restores normal focus-visible behaviour.
    if (!capturing) {
      interactionFocus.onKeyDown(false);
      return;
    }
    const target = event.target as HTMLElement;
    const passes =
      // Local shortcuts capture Tab; elsewhere it leaves, or moves through
      // the picker's parts, and leaving the field ends the recording.
      (!local &&
        event.key === "Tab" &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.metaKey) ||
      (pick !== null &&
        !hasModifier(event) &&
        (NAVIGATION.includes(event.key) ||
          (event.key === " " && target.closest("[data-modifier-toggle]"))));
    if (passes) {
      interactionFocus.onKeyDown(false);
      if (event.key === "Tab" && !pick) stop();
      return;
    }
    interactionFocus.onKeyDown(true);
    event.preventDefault();
    event.stopPropagation();
    if (event.nativeEvent.isComposing || event.repeat) return;
    if (event.key === "Escape") {
      cancel();
      return;
    }
    if (!ready) return;
    if (current && event.key === "Enter" && !hasModifier(event)) {
      take(current);
      return;
    }
    if (
      !single &&
      !local &&
      isClearable &&
      !hasModifier(event) &&
      (event.key === "Backspace" || event.key === "Delete")
    ) {
      commit(null);
      return;
    }
    const next = hotkeyFromEvent(event.nativeEvent, captureMode);
    if (single || local) {
      const conflict = next ? conflictFor(next) : null;
      if (next && conflict)
        setCaptureError(
          t("hotkey-field-local-taken", {
            action: conflict.label,
            keys: hotkeyKeys(next).map(keyLabel).join(" + "),
          }),
        );
      else if (next) commit(next);
      return;
    }
    if (next && !conflictFor(next)) {
      commit(next);
      return;
    }
    const held = next ? parseHotkey(next, isMac) : null;
    const code = held?.code ?? keyCodeFromEvent(event.nativeEvent);
    if (code) startPick(code, held?.modifiers);
  };

  return {
    buttonRef,
    captureError,
    capturing,
    commit,
    /** The combination in hand, if it is one of the chips. */
    current: suggestions.some((suggestion) => suggestion.value === current)
      ? current
      : null,
    feedback,
    interactionFocus,
    mouseProgress,
    pick,
    ready,
    /** The field's frame, which reads the keys and notices being left. */
    rootProps: {
      onBlur: (event: FocusEvent<HTMLDivElement>) => {
        interactionFocus.onBlur();
        if (!capturing || rootRef.current?.contains(event.relatedTarget))
          return;
        // Focus may be on its way to a part about to be drawn, such as the
        // chips a key brings; only a field left behind stops recording.
        // Focus left for good stays where it went.
        window.setTimeout(() => {
          if (rootRef.current?.contains(document.activeElement)) return;
          cancel();
          refocusRef.current = false;
        });
      },
      onKeyDownCapture,
      // A press on the picker between its controls keeps the keyboard in it.
      onMouseDown: (event: MouseEvent<HTMLDivElement>) => {
        if (pick && !(event.target as Element).closest("button, label"))
          event.preventDefault();
      },
      ref: rootRef,
    },
    setFeedback,
    setPick,
    /** The field pressed: starts recording, or stops it. */
    start: () => {
      setReady(false);
      setCaptureError(null);
      setListening(!capturing);
    },
    status,
    stop,
    suggestions,
    take,
  };
}
