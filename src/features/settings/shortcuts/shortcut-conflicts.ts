// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { hotkeysEqual } from "../../../lib/hotkey";

import { reservedUse } from "./reserved-shortcuts";

// What a combination would clash with, worked out here before anything is
// saved, from the same settings Rust checks them against, so the two
// cannot disagree. Keys only clash within a scope: a key that works while
// the ruler is open never meets one that works while text is showing.

/** Where a shortcut works: anywhere, or only while a tool is open. */
export type ShortcutScope =
  "annotate" | "anywhere" | "editor" | "ocr" | "ruler";

/** Whether a scope's keys are built in, shown but never set: the editor's
 * and the live Annotate overlay's. */
export const isFixedScope = (scope: ShortcutScope) =>
  scope === "editor" || scope === "annotate";

export type ShortcutConflict =
  /** Another of Screenwide's shortcuts has it; `label` names that one. */
  | { kind: "taken"; label: string }
  /** The system or most apps use it; `label` says for what. */
  | { by: "apps" | "system"; kind: "reserved"; label: string };

/** What the clash check needs of a shortcut. */
export type ConflictSubject = {
  id: string;
  /** A combination pressed together, or a key held (Glide's), which only
   * ever clashes with its own kind. */
  kind: "combo" | "hold";
  label: string;
  scope: ShortcutScope;
  value: string | null;
  /** A key built into a tool. Such keys never clash: where two share one,
   * what the tool has chosen decides between them, as the arrows nudge a
   * chosen layer and otherwise step the playhead. */
  isFixed?: boolean;
};

/** What taking `value` for `subject` would clash with, if anything: a
 * shortcut of its own scope first, then, for one that works anywhere, the
 * system's and every app's. */
export function conflictOf({
  isMac,
  shortcuts,
  subject,
  value,
}: {
  isMac: boolean;
  shortcuts: readonly ConflictSubject[];
  subject: Pick<ConflictSubject, "id" | "scope">;
  value: string;
}): ShortcutConflict | null {
  const taken = shortcuts.find(
    (other) =>
      other.kind === "combo" &&
      other.id !== subject.id &&
      other.scope === subject.scope &&
      hotkeysEqual(other.value, value, isMac),
  );
  if (taken) return { kind: "taken", label: taken.label };
  if (subject.scope !== "anywhere") return null;
  const reserved = reservedUse(value, isMac);
  return reserved
    ? { by: reserved.by, kind: "reserved", label: reserved.label() }
    : null;
}

/** Each shortcut that shares its combination with another of its scope,
 * with the others it shares it with. */
export function clashes<Subject extends ConflictSubject>(
  shortcuts: readonly Subject[],
  isMac: boolean,
): Map<string, Subject[]> {
  const found = new Map<string, Subject[]>();
  for (const shortcut of shortcuts) {
    if (shortcut.kind !== "combo" || !shortcut.value || shortcut.isFixed)
      continue;
    const others = shortcuts.filter(
      (other) =>
        other !== shortcut &&
        other.kind === "combo" &&
        !other.isFixed &&
        other.scope === shortcut.scope &&
        hotkeysEqual(other.value, shortcut.value, isMac),
    );
    if (others.length > 0) found.set(shortcut.id, others);
  }
  return found;
}
