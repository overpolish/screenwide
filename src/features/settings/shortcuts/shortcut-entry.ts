// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ConflictSubject } from "./shortcut-conflicts";
import type { HotkeyCaptureMode } from "../../../lib/hotkey";
import type { LucideIcon } from "lucide-react";

// One shortcut as the Shortcuts pane shows it: a row in the list and a key
// on the map. Most are set in their row; the editor's and Annotate's own keys
// are built in, and only shown.

type EntryBase = ConflictSubject & {
  group: string;
  icon: LucideIcon;
  /** Its feature is off, so the key does nothing for now. */
  isOff: boolean;
  description?: string;
  /** One of the system's keys, taken for the tool it names. */
  isSystem?: boolean;
};

/** A key built into a tool, shown but never set. */
export type FixedShortcutEntry = EntryBase & {
  /** Other keys that do the same, such as P beside Space for playing. */
  alternates: readonly string[];
  isFixed: true;
};

/** A shortcut set in its row. */
export type SettableShortcutEntry = EntryBase & {
  captureMode: HotkeyCaptureMode;
  isClearable: boolean;
  isDisabled: boolean;
  onChange: (value: string | null) => void;
  defaultValue?: string | null;
  isFixed?: false;
};

export type ShortcutEntry = FixedShortcutEntry | SettableShortcutEntry;

/** Every combination a shortcut answers to. */
export const shortcutValues = (entry: ShortcutEntry): string[] =>
  entry.value === null
    ? []
    : [entry.value, ...(entry.isFixed ? entry.alternates : [])];
