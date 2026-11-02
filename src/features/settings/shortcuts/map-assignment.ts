// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type HotkeyModifier,
  hotkeyValue,
  isGlobalChord,
} from "../../../lib/hotkey";

import { bestSuggestion, suggestShortcuts } from "./shortcut-suggestions";

import type { ShortcutConflict, ShortcutScope } from "./shortcut-conflicts";

// A key clicked on the keyboard map, with a shortcut chosen in the list,
// gives that shortcut the key under the latched modifiers, by the rules
// the field's suggestions follow: a shortcut that works anywhere needs a
// modifier besides Shift, so with none latched it takes the best free
// combination for the key; a clash or a reserved combination is refused
// rather than taken unseen, and is left for the field, where it can be
// chosen knowingly.

export type MapAssignment =
  | { kind: "assign"; modifiers: HotkeyModifier[]; value: string }
  | { conflict: ShortcutConflict; kind: "refuse"; value: string }
  /** No combination for the key is free. */
  | { kind: "none" };

export function assignFromMap({
  code,
  conflictOf,
  isMac,
  latched,
  shortcut,
}: {
  /** The key clicked, as a `KeyboardEvent.code`. */
  code: string;
  conflictOf: (value: string) => ShortcutConflict | null;
  isMac: boolean;
  latched: readonly HotkeyModifier[];
  shortcut: { kind: "combo" | "hold"; scope: ShortcutScope };
}): MapAssignment {
  // A held key is the key alone; the swap between Glide's keys happens
  // when it is saved.
  if (shortcut.kind === "hold")
    return { kind: "assign", modifiers: [], value: code };
  if (shortcut.scope === "anywhere" && !isGlobalChord(latched)) {
    const best = bestSuggestion(suggestShortcuts(code, { conflictOf, isMac }));
    return best
      ? { kind: "assign", modifiers: best.modifiers, value: best.value }
      : { kind: "none" };
  }
  const value = hotkeyValue(code, latched);
  const conflict = conflictOf(value);
  return conflict
    ? { conflict, kind: "refuse", value }
    : { kind: "assign", modifiers: [...latched], value };
}
