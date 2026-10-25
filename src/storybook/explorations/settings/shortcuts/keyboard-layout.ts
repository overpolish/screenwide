// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Modifier } from "./shortcut-model";

// A Mac keyboard's main block, US layout, in key widths. Every row adds up
// to 14.5 keys, so with each key as wide as its share and the gaps it spans,
// the rows end together. The arrow keys are left, an up and down pair
// stacked at half height, and right, as on a laptop.

export type KeyDef = {
  id: string;
  label: string;
  modifier?: Modifier;
  /** In keys; one when left out. */
  width?: number;
};

/** Keys whose id and label are the character printed on them, written as
 * one string with spaces between. */
const keys = (characters: string) =>
  characters
    .split(" ")
    .map((character) => ({ id: character, label: character }));

/** A modifier at an end of the bottom rows, named for its side, since a
 * held key such as Glide's can be one side only. */
const modifier = (
  key: `${Modifier}-${"left" | "right"}`,
  label: string,
  width: number,
): KeyDef => ({
  id: key,
  label,
  modifier: key.split("-")[0] as Modifier,
  width,
});

export const KEY_ROWS: KeyDef[][] = [
  [
    ...keys("` 1 2 3 4 5 6 7 8 9 0 - ="),
    { id: "delete", label: "delete", width: 1.5 },
  ],
  [
    { id: "tab", label: "tab", width: 1.5 },
    ...keys("q w e r t y u i o p [ ] \\"),
  ],
  [
    { id: "caps", label: "caps lock", width: 1.75 },
    ...keys("a s d f g h j k l ; '"),
    { id: "return", label: "return", width: 1.75 },
  ],
  [
    modifier("shift-left", "shift", 2.25),
    ...keys("z x c v b n m , . /"),
    modifier("shift-right", "shift", 2.25),
  ],
  [
    { id: "fn", label: "fn" },
    modifier("ctrl-left", "control", 1),
    modifier("opt-left", "option", 1),
    modifier("cmd-left", "command", 1.25),
    { id: "space", label: "", width: 5 },
    modifier("cmd-right", "command", 1.25),
    modifier("opt-right", "option", 1),
    { id: "left", label: "←" },
    { id: "updown", label: "" },
    { id: "right", label: "→" },
  ],
];

/** Keys across a row, for working out how wide one is. */
export const ROW_UNITS = 14.5;
