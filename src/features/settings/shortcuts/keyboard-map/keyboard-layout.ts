// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { HotkeyModifier } from "../../../../lib/hotkey";

// A laptop keyboard's main block, US layout, Mac or Windows, in key widths.
// Every row adds up to 14.5 keys, so the rows end together. Keys are named
// by `KeyboardEvent.code`, the same names a stored shortcut ends in, so a
// shortcut finds its key without a table between them. The arrows are
// left, an up and down pair stacked at half height, and right.

export type KeyDef = {
  code: string;
  /** A modifier, which the map latches rather than assigns. */
  modifier?: HotkeyModifier;
  /** For a key with a name rather than a character: the name `Keyboard`
   * and `keyLabel` know it by. */
  name?: string;
  /** In keys; one when left out. */
  width?: number;
};

/** The arrows stacked at half height, kept together as one column. */
type KeyColumn = { column: [KeyDef, KeyDef] };

type KeyRow = (KeyColumn | KeyDef)[];

const characters = (codes: string) =>
  codes.split(" ").map((code): KeyDef => ({ code }));
const letters = (row: string) =>
  row.split(" ").map((letter): KeyDef => ({ code: `Key${letter}` }));

const named = (code: string, name: string, width = 1): KeyDef => ({
  code,
  name,
  width,
});
// The modifier a key holds, from its code: `ShiftLeft` holds Shift.
const HELD: Record<string, HotkeyModifier> = {
  Alt: "Alt",
  Control: "Control",
  Meta: "Super",
  Shift: "Shift",
};
const modifier = (code: string, name: string, width = 1): KeyDef => ({
  code,
  modifier: HELD[code.replace(/(Left|Right)$/, "")],
  name,
  width,
});

const arrows: KeyRow = [
  named("ArrowLeft", "left"),
  { column: [named("ArrowUp", "up"), named("ArrowDown", "down")] },
  named("ArrowRight", "right"),
];

const mainBlock: KeyRow[] = [
  [
    ...characters(
      "Backquote Digit1 Digit2 Digit3 Digit4 Digit5 Digit6 Digit7 Digit8 Digit9 Digit0 Minus Equal",
    ),
    named("Backspace", "backspace", 1.5),
  ],
  [
    named("Tab", "tab", 1.5),
    ...letters("Q W E R T Y U I O P"),
    ...characters("BracketLeft BracketRight Backslash"),
  ],
  [
    named("CapsLock", "capslock", 1.75),
    ...letters("A S D F G H J K L"),
    ...characters("Semicolon Quote"),
    named("Enter", "enter", 1.75),
  ],
  [
    modifier("ShiftLeft", "shift", 2.25),
    ...letters("Z X C V B N M"),
    ...characters("Comma Period Slash"),
    modifier("ShiftRight", "shift", 2.25),
  ],
];

const MAC: KeyRow[] = [
  ...mainBlock,
  [
    named("Fn", "fn"),
    modifier("ControlLeft", "control"),
    modifier("AltLeft", "option"),
    modifier("MetaLeft", "command", 1.25),
    named("Space", "space", 5),
    modifier("MetaRight", "command", 1.25),
    modifier("AltRight", "option"),
    ...arrows,
  ],
];

const WINDOWS: KeyRow[] = [
  ...mainBlock,
  [
    modifier("ControlLeft", "ctrl"),
    named("Fn", "fn"),
    modifier("MetaLeft", "win"),
    modifier("AltLeft", "alt", 1.25),
    named("Space", "space", 5),
    modifier("AltRight", "alt", 1.25),
    modifier("ControlRight", "ctrl"),
    ...arrows,
  ],
];

export const keyboardRows = (isMac: boolean) => (isMac ? MAC : WINDOWS);
