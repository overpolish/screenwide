// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { Background, BackgroundPreset } from "./background";
import {
  backgroundSelection,
  type BackgroundSelection,
  type ChoiceMemory,
  customBackground,
  memoryAfter,
  NO_CHOICE_MEMORY,
  presetBackground,
} from "./background-choice";

/** The tile chosen, what Custom holds, and the presses that change them. */
export type BackgroundChoice = Omit<BackgroundSelection, "memory"> & {
  /** A change the editor made: it moves the choice to Custom, as the
   * background is now one of your own. */
  changeFromEditor: (background: Background) => void;
  /** A tile's own choice: it moves the choice off Custom. */
  choose: (background: Background) => void;
  choosePreset: (preset: BackgroundPreset, isKept: boolean) => void;
  /** Keeps what Custom holds through `save`. Custom empties, as the
   * background now has a tile of its own. */
  keep: (save: (background: Background) => void) => void;
  /** Brings back the last background of your own, or starts one as a
   * colour from the background on the canvas. */
  openCustom: () => void;
};

/**
 * Which background tile is chosen, what the Custom tile holds, and whether
 * the colours line is wanted, for the backgrounds on offer and the one the
 * canvas is wearing (`backgroundSelection`). The tiles and the colours are
 * laid out by whoever shows them.
 */
export function useBackgroundChoice({
  onChange,
  presets,
  savedPresets,
  value,
}: {
  onChange: (background: Background) => void;
  presets: BackgroundPreset[];
  savedPresets: BackgroundPreset[];
  value: Background;
}): BackgroundChoice {
  const [memory, setMemory] = useState<ChoiceMemory>(NO_CHOICE_MEMORY);
  const { memory: remembered, ...selection } = backgroundSelection({
    memory,
    presets,
    savedPresets,
    value,
  });
  if (remembered !== memory) setMemory(remembered);
  return {
    ...selection,
    changeFromEditor: (background) => {
      setMemory(memoryAfter.edit);
      onChange(background);
    },
    choose: (background) => {
      setMemory(memoryAfter.tile);
      onChange(background);
    },
    choosePreset: (preset, isKept) => {
      setMemory(memoryAfter.tile);
      onChange(presetBackground(preset, isKept));
    },
    keep: (save) => {
      if (!selection.custom) return;
      setMemory(memoryAfter.keep());
      save(selection.custom);
    },
    openCustom: () => {
      setMemory(memoryAfter.openCustom);
      const next = customBackground(selection, value);
      if (next) onChange(next);
    },
  };
}
