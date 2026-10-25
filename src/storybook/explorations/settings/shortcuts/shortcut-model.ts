// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { HotkeyCaptureMode } from "../../../../lib/hotkey";
import type { LucideIcon } from "lucide-react";

// Shortcuts as the keyboard map reads them: which key, under which
// modifiers. Stored shortcuts are accelerators ("CommandOrControl+Shift+KeyS",
// or "Super+Shift+KeyS" as a field captures one); the map names keys by
// what is printed on them, as its layout does.

export type Modifier = "cmd" | "ctrl" | "opt" | "shift";

/** The order a Mac menu writes modifiers in. */
export const MODIFIERS: Modifier[] = ["ctrl", "opt", "shift", "cmd"];

export const GLYPH: Record<Modifier, string> = {
  cmd: "⌘",
  ctrl: "⌃",
  opt: "⌥",
  shift: "⇧",
};

export type Combo = { key: string; mods: Modifier[] };

/** Where a shortcut works: anywhere, or only while a tool is open. Keys of
 * different scopes never clash, so the map shows one scope at a time. */
export type Scope = "anywhere" | "ocr" | "ruler";

export type ShortcutEntry = {
  captureMode: HotkeyCaptureMode;
  group: string;
  icon: LucideIcon;
  id: string;
  /** A key pressed with modifiers, or one held while dragging (Glide's). */
  kind: "combo" | "hold";
  label: string;
  onChange: (value: string | null) => void;
  scope: Scope;
  value: string | null;
  description?: string;
  /** Its tool is off, so the key does nothing for now. */
  isOff?: boolean;
};

const NAMED: Record<string, string> = {
  Backquote: "`",
  Backslash: "\\",
  Backspace: "delete",
  BracketLeft: "[",
  BracketRight: "]",
  CapsLock: "caps",
  Comma: ",",
  Enter: "return",
  Equal: "=",
  Minus: "-",
  Period: ".",
  Quote: "'",
  Semicolon: ";",
  Slash: "/",
  Space: "space",
  Tab: "tab",
};
const CODES = Object.fromEntries(
  Object.entries(NAMED).map(([code, key]) => [key, code]),
);

/** The key a `KeyboardEvent.code` stands for, as the map names it. */
export const keyOfCode = (code: string) =>
  code.startsWith("Key")
    ? code.slice(3).toLowerCase()
    : code.startsWith("Digit")
      ? code.slice(5)
      : (NAMED[code] ?? code.replace(/^Arrow/, "").toLowerCase());

const codeOfKey = (key: string) =>
  /^[a-z]$/.test(key)
    ? `Key${key.toUpperCase()}`
    : /^\d$/.test(key)
      ? `Digit${key}`
      : (CODES[key] ?? key);

const modifierOf = (part: string): Modifier | null => {
  if (["Command", "CommandOrControl", "Meta", "Super"].includes(part))
    return "cmd";
  if (part === "Control") return "ctrl";
  if (part === "Alt") return "opt";
  if (part === "Shift") return "shift";
  return null;
};

export function parseCombo(value: string | null): Combo | null {
  if (!value) return null;
  const parts = value.split("+");
  const code = parts.pop() ?? "";
  const mods = new Set(parts.map(modifierOf));
  return {
    key: keyOfCode(code),
    mods: MODIFIERS.filter((modifier) => mods.has(modifier)),
  };
}

export const comboValue = ({ key, mods }: Combo) =>
  [
    ...mods.map(
      (modifier) =>
        ({
          cmd: "CommandOrControl",
          ctrl: "Control",
          opt: "Alt",
          shift: "Shift",
        })[modifier],
    ),
    codeOfKey(key),
  ].join("+");

/** The map key a held control is: a modifier on its side, or any key. */
export function holdKey(value: string | null) {
  if (!value) return null;
  const side = /^(Meta|Control|Alt|Shift)(Left|Right)$/.exec(value);
  if (!side) return keyOfCode(value);
  const modifier = modifierOf(side[1]) ?? "cmd";
  return `${modifier}-${side[2].toLowerCase()}`;
}

const KEY_GLYPH: Record<string, string> = {
  delete: "⌫",
  down: "↓",
  left: "←",
  return: "↩",
  right: "→",
  space: "Space",
  tab: "⇥",
  up: "↑",
};

export const keyText = (key: string) =>
  KEY_GLYPH[key] ?? (key.length === 1 ? key.toUpperCase() : key);

export const comboText = (combo: Combo) =>
  combo.mods.map((modifier) => GLYPH[modifier]).join("") + keyText(combo.key);

export const heldText = (held: Iterable<Modifier>) => {
  const set = new Set(held);
  return MODIFIERS.filter((modifier) => set.has(modifier))
    .map((modifier) => GLYPH[modifier])
    .join("");
};

export const sameMods = (mods: Modifier[], held: ReadonlySet<Modifier>) =>
  mods.length === held.size && mods.every((modifier) => held.has(modifier));

export type Reserved = Combo & { by: "apps" | "macOS"; label: string };

const reserve =
  (by: Reserved["by"]) =>
  (mods: string, key: string, label: string): Reserved => ({
    by,
    key,
    label,
    mods: MODIFIERS.filter((modifier) => mods.split(" ").includes(modifier)),
  });
const system = reserve("macOS");
const apps = reserve("apps");

/** Combinations a shortcut that works anywhere should not take: the
 * system's own, and the ones every app uses, which a global shortcut would
 * take from them. */
export const RESERVED: Reserved[] = [
  system("cmd", "space", "Spotlight"),
  system("ctrl", "space", "Switch input source"),
  system("ctrl cmd", "space", "Emoji and symbols"),
  system("cmd", "tab", "Switch apps"),
  system("cmd", "`", "Switch windows"),
  system("shift cmd", "3", "Screenshot of the screen"),
  system("shift cmd", "4", "Screenshot of an area"),
  system("shift cmd", "5", "Screenshot and recording"),
  system("ctrl cmd", "q", "Lock screen"),
  apps("cmd", "q", "Quit"),
  apps("cmd", "w", "Close window"),
  apps("cmd", "h", "Hide"),
  apps("cmd", "m", "Minimise"),
  apps("cmd", "c", "Copy"),
  apps("cmd", "v", "Paste"),
  apps("cmd", "x", "Cut"),
  apps("cmd", "z", "Undo"),
  apps("shift cmd", "z", "Redo"),
  apps("cmd", "a", "Select all"),
  apps("cmd", "s", "Save"),
  apps("cmd", "f", "Find"),
  apps("cmd", "n", "New"),
  apps("cmd", "o", "Open"),
  apps("cmd", "p", "Print"),
  apps("cmd", "t", "New tab"),
  apps("cmd", ",", "Settings"),
];

/** Entries that share a combination with another in their scope. */
export function clashes(entries: ShortcutEntry[]) {
  const found = new Map<string, ShortcutEntry[]>();
  for (const entry of entries) {
    if (entry.kind !== "combo") continue;
    const combo = parseCombo(entry.value);
    if (!combo) continue;
    const id = `${entry.scope} ${comboText(combo)}`;
    found.set(id, [...(found.get(id) ?? []), entry]);
  }
  const result = new Map<string, ShortcutEntry[]>();
  for (const group of found.values()) {
    if (group.length < 2) continue;
    for (const entry of group)
      result.set(
        entry.id,
        group.filter((other) => other !== entry),
      );
  }
  return result;
}
