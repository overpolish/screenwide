// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Beat } from "../glide-playback";

import type { GlidePickerScreen } from "./glide-picker-arrange";
import type { GlidePickerState } from "./glide-picker-types";

// The desktops the picker's stories show, and the picks they play.

const row = (count: number): GlidePickerScreen[] =>
  Array.from({ length: count }, (_, index) => ({ x: index, y: 0 }));

export const SPACES = row(4);
export const MANY_SPACES = row(8);

/** A laptop between two 1920×1080 displays, their bottoms level with its
 * own, by their centres. The rule keeps centres' distances, so displays this
 * evenly spaced sit a gap apart; mixed sizes spread out further. */
export const DISPLAYS: GlidePickerScreen[] = [
  { x: -960, y: 442 },
  { x: 756, y: 491 },
  { x: 2472, y: 442 },
];

export const HIDDEN: GlidePickerState = {
  isShown: false,
  origin: 0,
  selected: 0,
};

/** The window on one screen, about to go to another. */
export const ELSEWHERE: GlidePickerState = {
  isShown: true,
  origin: 1,
  selected: 5,
};

export const picks = {
  /** From the laptop to the display on its right, a rest, across to the one
   * on its left, a rest, and away. */
  displays: [
    {
      after: 700,
      icon: false,
      state: { isShown: true, origin: 1, selected: 1 },
    },
    { after: 150, icon: true },
    { after: 650, state: { selected: 2 } },
    { after: 500, rest: true },
    { after: 800, state: { selected: 1 } },
    { after: 400, state: { selected: 0 } },
    { after: 500, rest: true },
    { after: 900, state: { isShown: false } },
  ],
  /** From the second Space to the last, a rest, one back, a rest, and
   * away. */
  spaces: [
    {
      after: 700,
      icon: false,
      state: { isShown: true, origin: 1, selected: 1 },
    },
    { after: 150, icon: true },
    { after: 650, state: { selected: 2 } },
    { after: 450, state: { selected: 3 } },
    { after: 500, rest: true },
    { after: 800, state: { selected: 2 } },
    { after: 500, rest: true },
    { after: 900, state: { isShown: false } },
  ],
} satisfies Record<string, Beat<GlidePickerState>[]>;

export type GlidePickerPick = keyof typeof picks;
