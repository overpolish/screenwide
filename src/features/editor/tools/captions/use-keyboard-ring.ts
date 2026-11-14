// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type FocusEvent, type KeyboardEvent, useState } from "react";

/** Keys that never move through the words: Space only plays and pauses,
 * and the modifiers are held for presses, as Command is to hide words. */
const QUIET_KEYS = new Set([" ", "Alt", "Control", "Meta", "Shift"]);

/**
 * Whether a list of words shows the ring on the word its keys are on: once
 * it is reached with Tab or a key is used in it, not when a press put focus
 * there, and not for Space or a modifier held. The browser's
 * `:focus-visible` would show it for any key, Space included.
 */
export function useKeyboardRing() {
  const [isShown, setIsShown] = useState(false);
  /** Spread on the list: it carries `data-ring` while the ring shows. */
  return {
    "data-ring": isShown || undefined,
    onBlur: () => {
      setIsShown(false);
    },
    onFocus: (event: FocusEvent<HTMLElement>) => {
      setIsShown(event.currentTarget.matches(":focus-visible"));
    },
    onKeyDownCapture: (event: KeyboardEvent<HTMLElement>) => {
      if (!QUIET_KEYS.has(event.key)) setIsShown(true);
    },
    onPointerDownCapture: () => {
      setIsShown(false);
    },
  };
}
