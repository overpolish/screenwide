// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isMacHost } from "../../../lib/platform";

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

/** Compare persisted shortcuts without treating equivalent aliases/order as edits. */
function normalizeHotkey(
  value: string | null,
  isMac = isMacHost,
): string | null {
  if (value === null) return null;
  const parts = value.split("+");
  const key = parts[parts.length - 1] ?? "";
  const modifiers = parts
    .slice(0, -1)
    .map((part) =>
      part === "CommandOrControl"
        ? isMac
          ? "Super"
          : "Control"
        : part === "Command" || part === "Meta"
          ? "Super"
          : part === "Control"
            ? "Control"
            : part === "Alt"
              ? "Alt"
              : part === "Shift"
                ? "Shift"
                : part,
    )
    .sort();
  return [...modifiers, key].join("+");
}

export function hotkeysEqual(
  value: string | null,
  defaultValue: string | null,
  isMac = isMacHost,
): boolean {
  return normalizeHotkey(value, isMac) === normalizeHotkey(defaultValue, isMac);
}

export function hotkeyFromEvent(
  event: KeyEvent,
  mode: HotkeyCaptureMode = "shortcut",
): string | null {
  if (
    event.repeat ||
    event.isComposing ||
    !event.code ||
    event.code === "Unidentified" ||
    ["Dead", "Process"].includes(event.key)
  )
    return null;
  if (mode === "single-control") return event.code;
  if (["Alt", "AltGraph", "Control", "Meta", "Shift"].includes(event.key))
    return null;
  const modifiers = [
    event.metaKey ? "Super" : null,
    event.ctrlKey ? "Control" : null,
    event.altKey ? "Alt" : null,
    event.shiftKey ? "Shift" : null,
  ].filter(Boolean);
  if (!modifiers.length) return mode === "local-shortcut" ? event.code : null;
  return [...modifiers, event.code].join("+");
}

// The order each platform names modifiers in: a Mac menu draws ⌃⌥⇧⌘, and
// WinUI writes an accelerator as Ctrl, Alt, Win, Shift. The key comes last.
const MAC_MODIFIER_ORDER = ["Control", "Option", "Shift", "Command"];
const WINDOWS_MODIFIER_ORDER = ["Control", "Alt", "Win", "Shift"];

/** The keys of a shortcut, each as a name `Keyboard` and `keyLabel` know:
 * the platform's modifiers in its order, then the key a code stands for or a
 * mouse control. */
export function hotkeyKeys(value: string | null, isMac = isMacHost): string[] {
  if (!value) return [];
  const order = isMac ? MAC_MODIFIER_ORDER : WINDOWS_MODIFIER_ORDER;
  const rank = (key: string) => {
    const index = order.indexOf(key);
    return index === -1 ? order.length : index;
  };
  return value
    .split("+")
    .map((key) => {
      key = key.replace(/^(Meta|Control|Alt|Shift)(Left|Right)$/, "$1");
      if (key === "CommandOrControl") return isMac ? "Command" : "Control";
      if (key === "Super" || key === "Meta" || key === "Command")
        return isMac ? "Command" : "Win";
      if (key === "Alt") return isMac ? "Option" : "Alt";
      if (key.startsWith("Key")) return key.slice(3);
      if (key.startsWith("Digit")) return key.slice(5);
      return key.replace(/^Arrow/, "").replace(/^Numpad/, "Num ");
    })
    .sort((a, b) => rank(a) - rank(b));
}

export const mouseControlFromButton = (button: number): string | null =>
  ({ 1: "MouseMiddle", 3: "MouseBack", 4: "MouseForward" })[button] ?? null;
