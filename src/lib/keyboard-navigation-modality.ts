// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getInteractionModality,
  setInteractionModality,
} from "react-aria/private/interactions/useFocusVisible";

const navigationKeys = new Set([
  "Tab",
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "End",
  "Home",
  "PageDown",
  "PageUp",
]);
type InteractionModality = "keyboard" | "pointer" | "virtual";

const isNavigationKey = (event: KeyboardEvent) =>
  navigationKeys.has(event.key) &&
  !event.ctrlKey &&
  !event.metaKey &&
  !event.altKey;

/**
 * React Aria promotes every key to keyboard modality. Keep that behavior for
 * navigation, while restoring the modality that was present for ordinary
 * shortcuts and text input. Window capture snapshots before React Aria's
 * document capture listener; document capture applies the result afterward.
 */
export function installKeyboardNavigationModality() {
  // A fresh webview has no modality yet, which React Aria treats as visible.
  // Native window activation can focus a control before any web input arrives.
  // Start without a ring, preserving any interaction already observed.
  if (getInteractionModality() == null) setInteractionModality("pointer");

  const snapshots = new WeakMap<KeyboardEvent, InteractionModality>();
  const capture = (event: KeyboardEvent) => {
    snapshots.set(event, getInteractionModality() ?? "pointer");
  };
  const restore = (event: KeyboardEvent) => {
    if (isNavigationKey(event)) {
      setInteractionModality("keyboard");
      return;
    }
    setInteractionModality(snapshots.get(event) ?? "pointer");
  };

  for (const type of ["keydown", "keyup"] as const) {
    window.addEventListener(type, capture, { capture: true });
    document.addEventListener(type, restore, { capture: true });
  }
  return () => {
    for (const type of ["keydown", "keyup"] as const) {
      window.removeEventListener(type, capture, { capture: true });
      document.removeEventListener(type, restore, { capture: true });
    }
  };
}
