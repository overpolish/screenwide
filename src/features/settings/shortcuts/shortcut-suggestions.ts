// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type HotkeyModifier,
  hotkeysEqual,
  hotkeyValue,
  isGlobalChord,
} from "../../../lib/hotkey";

import type { ShortcutConflict } from "./shortcut-conflicts";

// The combinations offered for a key pressed alone, best first, so a
// shortcut is set by pressing one key and picking, not by holding three.
//
// On a Mac, ⌘⇧ is what apps' global shortcuts use, then ⌃⌥, which no app
// menu takes. ⌥ with only ⇧ is left out: it types accented letters. On
// Windows, Ctrl+Shift comes first; Ctrl+Alt is AltGr on many keyboards,
// which types characters such as € and @, so it comes last. The Windows
// key is never offered: Windows keeps those combinations.
const MAC_RANKING: HotkeyModifier[][] = [
  ["Super", "Shift"],
  ["Control", "Alt"],
  ["Super", "Alt"],
  ["Control", "Shift"],
  ["Super", "Control"],
  ["Super", "Alt", "Shift"],
  ["Control", "Alt", "Shift"],
];
const WINDOWS_RANKING: HotkeyModifier[][] = [
  ["Control", "Shift"],
  ["Alt", "Shift"],
  ["Control", "Alt", "Shift"],
  ["Control", "Alt"],
];

/** How many combinations a key offers before any it adds to have a free
 * one. */
const SHOWN_SUGGESTIONS = 4;

export type ShortcutSuggestion = {
  conflict: ShortcutConflict | null;
  modifiers: HotkeyModifier[];
  value: string;
};

/** The combinations offered for `code`: the best few, in order, each with
 * what it would clash with. When all of those clash, the first free one
 * further down is added, so there is always one to take if any is free.
 * `include` adds combinations already in hand, such as one held down or
 * built from the modifier toggles, so they can be seen and chosen too. */
export function suggestShortcuts(
  code: string,
  {
    conflictOf,
    include = [],
    isMac,
  }: {
    conflictOf: (value: string) => ShortcutConflict | null;
    isMac: boolean;
    include?: readonly (readonly HotkeyModifier[])[];
  },
): ShortcutSuggestion[] {
  const suggest = (modifiers: HotkeyModifier[]): ShortcutSuggestion => {
    const value = hotkeyValue(code, modifiers);
    return { conflict: conflictOf(value), modifiers, value };
  };
  const ranked = (isMac ? MAC_RANKING : WINDOWS_RANKING).map(suggest);
  const shown = ranked.slice(0, SHOWN_SUGGESTIONS);
  if (!shown.some(({ conflict }) => !conflict)) {
    const free = ranked
      .slice(SHOWN_SUGGESTIONS)
      .find(({ conflict }) => !conflict);
    if (free) shown.push(free);
  }
  for (const modifiers of include) {
    if (!isGlobalChord(modifiers)) continue;
    const value = hotkeyValue(code, modifiers);
    if (!shown.some((shownOne) => hotkeysEqual(shownOne.value, value, isMac)))
      shown.push(suggest([...modifiers]));
  }
  return shown;
}

/** The one a key press offers first: the best free combination. A clash
 * is never the default; it can only be chosen by moving to it. */
export const bestSuggestion = (suggestions: readonly ShortcutSuggestion[]) =>
  suggestions.find(({ conflict }) => !conflict) ?? null;

/** The combination a platform's global shortcuts use most, which the
 * keyboard map starts latched to. */
export const preferredModifiers = (isMac: boolean) =>
  (isMac ? MAC_RANKING : WINDOWS_RANKING)[0];
