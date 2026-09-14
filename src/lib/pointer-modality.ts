// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getInteractionModality,
  setInteractionModality,
} from "react-aria/private/interactions/useFocusVisible";

import { installKeyboardNavigationModality } from "./keyboard-navigation-modality";

/** A window focus this soon after a press belongs to that press. */
const PRESS_FOCUS_WINDOW_MS = 1000;

/**
 * React Aria shows focus rings when the last input it observed was keyboard,
 * and it can treat restored element focus as virtual focus, which shows
 * rings too. The recording panels are non-activating
 * windows that blur whenever another panel or a shortcut takes over, so a
 * mouse press that re-activates one lit up the pressed button. Any mouse
 * press declares pointer modality, and a window focus that follows a press
 * restates it after React Aria's own handler has run. Native webview hiding
 * can restore focus long after that press, sometimes to a different control.
 * Preserve pointer modality through that hidden-to-visible transition. Outside
 * native restoration, new focus follows React Aria's normal rules.
 */
export function installPointerModalityGuard() {
  installKeyboardNavigationModality();
  let lastPressAt = Number.NEGATIVE_INFINITY;
  let pointerFocusBeforeBlur: Element | null = null;
  // Precreated windows start hidden without emitting a hide notification.
  let pointerBeforeHide =
    document.visibilityState === "hidden" &&
    getInteractionModality() === "pointer";
  let pointerEscapeReleaseTarget: Element | null = null;
  let inputVersion = 0;
  const declarePointer = () => {
    inputVersion++;
    pointerFocusBeforeBlur = null;
    pointerBeforeHide = false;
    pointerEscapeReleaseTarget = null;
    lastPressAt = performance.now();
    setInteractionModality("pointer");
  };
  for (const type of ["pointerdown", "mousedown"] as const) {
    document.addEventListener(type, declarePointer, { capture: true });
  }
  document.addEventListener(
    "keydown",
    () => {
      inputVersion++;
      lastPressAt = Number.NEGATIVE_INFINITY;
      pointerFocusBeforeBlur = null;
      pointerBeforeHide = false;
      pointerEscapeReleaseTarget = null;
    },
    { capture: true },
  );
  document.addEventListener(
    "keyup",
    (event) => {
      const target = pointerEscapeReleaseTarget;
      pointerEscapeReleaseTarget = null;
      if (event.key !== "Escape") return;
      // Native dismissal consumes Escape-down, then restores this window
      // before Escape-up. React Aria also changes modality on keyup. Restate
      // pointer modality after its listener, without swallowing the event.
      if (target && document.hasFocus() && document.activeElement === target)
        setInteractionModality("pointer");
    },
    { capture: true },
  );
  window.addEventListener("blur", () => {
    pointerFocusBeforeBlur =
      getInteractionModality() === "pointer" ? document.activeElement : null;
    pointerEscapeReleaseTarget = pointerFocusBeforeBlur;
  });
  window.addEventListener(
    "focus",
    (event) => {
      if (event.target === window || event.target === document) return;
      if (event.target !== pointerEscapeReleaseTarget)
        pointerEscapeReleaseTarget = null;
      // WebKit focuses the control again after visibility flips, but before
      // visibilitychange. Keep the snapshot until that notification arrives.
      const restoringHiddenPointer = pointerBeforeHide;
      const restoredPointerFocus =
        restoringHiddenPointer || event.target === pointerFocusBeforeBlur;
      if (!restoredPointerFocus || document.visibilityState !== "hidden")
        pointerFocusBeforeBlur = null;
      // Run after React Aria's capture listener, before React handles focus.
      if (restoredPointerFocus) setInteractionModality("pointer");
    },
    { capture: true },
  );
  window.addEventListener("focus", () => {
    const restoredElement = pointerFocusBeforeBlur;
    const restoringHiddenPointer = pointerBeforeHide;
    if (
      !restoringHiddenPointer &&
      !restoredElement &&
      performance.now() - lastPressAt > PRESS_FOCUS_WINDOW_MS
    )
      return;
    const version = inputVersion;
    // After React Aria's focus handler, which runs in the same task.
    setTimeout(() => {
      if (
        document.visibilityState !== "hidden" &&
        pointerFocusBeforeBlur === restoredElement
      )
        pointerFocusBeforeBlur = null;
      if (inputVersion !== version) return;
      if (
        !restoringHiddenPointer &&
        restoredElement &&
        document.activeElement !== restoredElement
      )
        return;
      setInteractionModality("pointer");
    }, 0);
  });
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") {
      pointerBeforeHide = getInteractionModality() === "pointer";
      return;
    }
    // WKWebView restoration emits multiple element focus events while the
    // document is still hidden. Preserve the snapshot through all of them.
    if (
      pointerBeforeHide ||
      (pointerFocusBeforeBlur &&
        document.activeElement === pointerFocusBeforeBlur)
    )
      setInteractionModality("pointer");
    pointerFocusBeforeBlur = null;
    pointerBeforeHide = false;
  });
}
