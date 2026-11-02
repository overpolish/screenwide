// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isWindows } from "./platform";

type KeyEvent = Pick<
  KeyboardEvent,
  | "key"
  | "code"
  | "metaKey"
  | "ctrlKey"
  | "altKey"
  | "shiftKey"
  | "repeat"
  | "isComposing"
>;

export type HotkeyCaptureMode =
  "shortcut" | "single-control" | "local-shortcut";

/** A modifier as a stored shortcut names it: `Super` is ⌘ on a Mac and the
 * Windows key on Windows. */
export type HotkeyModifier = "Alt" | "Control" | "Shift" | "Super";

/** The order a recorded shortcut writes its modifiers in. */
const HOTKEY_MODIFIERS: HotkeyModifier[] = ["Super", "Control", "Alt", "Shift"];

// The skin decides, not the host, so a story can show Windows on a Mac; in
// the app the two are the same.
const isMacSkin = () => !isWindows();

const modifierOf = (part: string, isMac: boolean): HotkeyModifier | null => {
  if (part === "CommandOrControl") return isMac ? "Super" : "Control";
  if (part === "Command" || part === "Meta" || part === "Super") return "Super";
  if (part === "Control" || part === "Alt" || part === "Shift") return part;
  return null;
};

export type ParsedHotkey = { code: string; modifiers: HotkeyModifier[] };

/** A stored shortcut as its key's code and its modifiers, aliases resolved
 * for the platform (`CommandOrControl` is ⌘ on a Mac, Ctrl on Windows). */
export function parseHotkey(
  value: string | null,
  isMac = isMacSkin(),
): ParsedHotkey | null {
  if (!value) return null;
  const parts = value.split("+");
  const code = parts.pop() ?? "";
  const named = new Set(parts.map((part) => modifierOf(part, isMac)));
  return {
    code,
    modifiers: HOTKEY_MODIFIERS.filter((modifier) => named.has(modifier)),
  };
}

/** A shortcut written as a recorded one is, whatever order the modifiers
 * come in. */
export const hotkeyValue = (
  code: string,
  modifiers: readonly HotkeyModifier[],
): string =>
  [
    ...HOTKEY_MODIFIERS.filter((modifier) => modifiers.includes(modifier)),
    code,
  ].join("+");

/** Whether a shortcut that works anywhere may use these modifiers. Shift
 * alone does not count: Shift and a letter types a capital, and a shortcut
 * that works anywhere would take it from every text field. */
export const isGlobalChord = (modifiers: readonly HotkeyModifier[]) =>
  modifiers.some((modifier) => modifier !== "Shift");

/** Compare persisted shortcuts without treating equivalent aliases/order as edits. */
function normalizeHotkey(value: string | null, isMac: boolean): string | null {
  const parsed = parseHotkey(value, isMac);
  return parsed ? hotkeyValue(parsed.code, parsed.modifiers) : null;
}

export function hotkeysEqual(
  value: string | null,
  defaultValue: string | null,
  isMac = isMacSkin(),
): boolean {
  return normalizeHotkey(value, isMac) === normalizeHotkey(defaultValue, isMac);
}

const MODIFIER_KEYS = ["Alt", "AltGraph", "Control", "Meta", "Shift"];

/** The code of the key pressed, its modifiers aside; null for a modifier
 * alone, a repeat, or a key still being composed. */
export function keyCodeFromEvent(event: KeyEvent): string | null {
  if (
    event.repeat ||
    event.isComposing ||
    !event.code ||
    event.code === "Unidentified" ||
    ["Dead", "Process"].includes(event.key) ||
    MODIFIER_KEYS.includes(event.key)
  )
    return null;
  return event.code;
}

export function hotkeyFromEvent(
  event: KeyEvent,
  mode: HotkeyCaptureMode = "shortcut",
): string | null {
  if (mode === "single-control") {
    if (
      event.repeat ||
      event.isComposing ||
      !event.code ||
      event.code === "Unidentified" ||
      ["Dead", "Process"].includes(event.key)
    )
      return null;
    return event.code;
  }
  const code = keyCodeFromEvent(event);
  if (!code) return null;
  const modifiers = HOTKEY_MODIFIERS.filter(
    (modifier) =>
      ({
        Alt: event.altKey,
        Control: event.ctrlKey,
        Shift: event.shiftKey,
        Super: event.metaKey,
      })[modifier],
  );
  if (mode === "local-shortcut") return hotkeyValue(code, modifiers);
  return isGlobalChord(modifiers) ? hotkeyValue(code, modifiers) : null;
}

// The order each platform names modifiers in: a Mac menu draws ⌃⌥⇧⌘, and
// WinUI writes an accelerator as Ctrl, Alt, Win, Shift. The key comes last.
const MAC_MODIFIER_ORDER = ["Control", "Option", "Shift", "Command"];
const WINDOWS_MODIFIER_ORDER = ["Control", "Alt", "Win", "Shift"];

/** The modifiers in the order the platform names them, as `hotkeyKeys`
 * would list them. */
export const platformModifiers = (isMac = isMacSkin()): HotkeyModifier[] =>
  isMac
    ? ["Control", "Alt", "Shift", "Super"]
    : ["Control", "Alt", "Super", "Shift"];

// Punctuation is named by what is printed on it, as a menu shows it.
const CHARACTERS: Record<string, string> = {
  Backquote: "`",
  Backslash: "\\",
  BracketLeft: "[",
  BracketRight: "]",
  Comma: ",",
  Equal: "=",
  Minus: "-",
  Period: ".",
  Quote: "'",
  Semicolon: ";",
  Slash: "/",
};

/** The name `Keyboard` and `keyLabel` know for one part of a shortcut: a
 * modifier, or the key a code stands for. */
export function hotkeyKey(part: string, isMac = isMacSkin()): string {
  const key = part.replace(/^(Meta|Control|Alt|Shift)(Left|Right)$/, "$1");
  if (key === "CommandOrControl") return isMac ? "Command" : "Control";
  if (key === "Super" || key === "Meta" || key === "Command")
    return isMac ? "Command" : "Win";
  if (key === "Alt") return isMac ? "Option" : "Alt";
  if (key.startsWith("Key")) return key.slice(3);
  if (key.startsWith("Digit")) return key.slice(5);
  return (
    CHARACTERS[key] ?? key.replace(/^Arrow/, "").replace(/^Numpad/, "Num ")
  );
}

/** The keys of a shortcut, each as a name `Keyboard` and `keyLabel` know:
 * the platform's modifiers in its order, then the key a code stands for or a
 * mouse control. */
export function hotkeyKeys(
  value: string | null,
  isMac = isMacSkin(),
): string[] {
  if (!value) return [];
  const order = isMac ? MAC_MODIFIER_ORDER : WINDOWS_MODIFIER_ORDER;
  const rank = (key: string) => {
    const index = order.indexOf(key);
    return index === -1 ? order.length : index;
  };
  return value
    .split("+")
    .map((key) => hotkeyKey(key, isMac))
    .sort((a, b) => rank(a) - rank(b));
}

export const mouseControlFromButton = (button: number): string | null =>
  ({ 1: "MouseMiddle", 3: "MouseBack", 4: "MouseForward" })[button] ?? null;
