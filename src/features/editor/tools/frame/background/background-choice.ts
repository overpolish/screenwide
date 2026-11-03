// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Background, BackgroundPreset, sameBackground } from "./background";
import { randomMeshForGenerator } from "./background-random";

/** Where a colour of your own starts when the background in hand gives
 * none: a picture. */
const DEFAULT_CUSTOM_COLOR = "#171717";

/**
 * What the picker remembers beyond the background on the canvas: whether
 * Custom was chosen by hand or by an edit, which holds the choice on Custom
 * even when what was built happens to match a tile, so the controls that
 * made it do not vanish under the hand using them; and the last background
 * of your own, so it outlives other tiles tried.
 */
export type ChoiceMemory = {
  built: Background | null;
  isCustomOpen: boolean;
};

export const NO_CHOICE_MEMORY: ChoiceMemory = {
  built: null,
  isCustomOpen: false,
};

/** How each press changes what the picker remembers. */
export const memoryAfter = {
  /** An edit makes the background one of your own. */
  edit: (memory: ChoiceMemory): ChoiceMemory => ({
    ...memory,
    isCustomOpen: true,
  }),
  /** Keeping gives the background a tile of its own, and Custom empties. */
  keep: (): ChoiceMemory => NO_CHOICE_MEMORY,
  /** Custom pressed. */
  openCustom: (memory: ChoiceMemory): ChoiceMemory => ({
    ...memory,
    isCustomOpen: true,
  }),
  /** Any other tile moves the choice off Custom. */
  tile: (memory: ChoiceMemory): ChoiceMemory => ({
    ...memory,
    isCustomOpen: false,
  }),
};

/** The picker as the background on the canvas and its memory make it. */
export type BackgroundSelection = {
  /** Whether Custom holds a background not kept yet, which a right click
   * there keeps. */
  canKeep: boolean;
  /** What Custom shows: the background of your own on the canvas, or the
   * last one built, or nothing. */
  custom: Background | null;
  /** Whether the background on the canvas is one of your own. */
  isCustom: boolean;
  /** The memory as of this background: it takes in a background of your
   * own while it is on the canvas. Stored back whenever it changes. */
  memory: ChoiceMemory;
  selectedId: string;
  /** Whether the colours line is wanted: for a background of your own, and
   * for any mesh, whose colours are set there. */
  showEditor: boolean;
};

/**
 * Which tile is chosen and what Custom holds. A tile stands for exactly what
 * it paints: a built-in mesh is its generator under its own colours, a kept
 * one is what was kept. Anything else is a background of your own, and
 * Custom is chosen while it is on the canvas.
 */
export function backgroundSelection({
  memory,
  presets,
  savedPresets,
  value,
}: {
  memory: ChoiceMemory;
  presets: BackgroundPreset[];
  savedPresets: BackgroundPreset[];
  value: Background;
}): BackgroundSelection {
  const isKept = (background: Background) =>
    savedPresets.some((preset) =>
      sameBackground(preset.background, background),
    );
  const matched =
    savedPresets.find((preset) => sameBackground(preset.background, value)) ??
    presets.find((preset) => sameBackground(preset.background, value));
  // A picture always has a tile of its own once picked, so it is never one
  // of your own.
  const isCustom = value.kind !== "image" && (memory.isCustomOpen || !matched);
  const remembered =
    isCustom && memory.built !== value ? { ...memory, built: value } : memory;
  // Once kept, a background has a tile of its own, and Custom lets it go,
  // however late the kept tile arrives.
  const custom = isCustom
    ? value
    : remembered.built && !isKept(remembered.built)
      ? remembered.built
      : null;
  return {
    // A built-in solid is a tile already, so it is not kept a second time.
    canKeep:
      custom !== null &&
      !isKept(custom) &&
      !(
        custom.kind === "solid" &&
        presets.some((preset) => sameBackground(preset.background, custom))
      ),
    custom,
    isCustom,
    memory: remembered,
    selectedId: isCustom ? "custom" : (matched?.id ?? "custom"),
    showEditor: isCustom || value.kind === "mesh",
  };
}

/** What a tile puts on the canvas. A built-in mesh draws its generator
 * under its own colours and a seed made on the spot, so pressing the chosen
 * tile again rolls a new picture; a kept mesh comes back exactly as it was
 * kept. */
export const presetBackground = (
  preset: BackgroundPreset,
  isKept: boolean,
): Background =>
  isKept || preset.background.kind !== "mesh"
    ? preset.background
    : randomMeshForGenerator(preset.background.generator);

/** What pressing Custom puts on the canvas: nothing while a background of
 * your own is already there, otherwise the last one built, or a colour
 * started from the background on the canvas: its own, a mesh's first, or a
 * dark grey under a picture. */
export function customBackground(
  selection: Pick<BackgroundSelection, "custom" | "isCustom">,
  value: Background,
): Background | null {
  if (selection.isCustom) return null;
  if (selection.custom) return selection.custom;
  const color =
    value.kind === "solid"
      ? value.color
      : value.kind === "mesh"
        ? (value.colors[0] ?? DEFAULT_CUSTOM_COLOR)
        : DEFAULT_CUSTOM_COLOR;
  return { color, kind: "solid" };
}
