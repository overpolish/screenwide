// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  backgroundSelection,
  customBackground,
  memoryAfter,
  NO_CHOICE_MEMORY,
  presetBackground,
} from "./background-choice";
import { backgroundGenerator } from "./background-generators";

import type { Background, BackgroundPreset } from "./background";

const auroraColors = backgroundGenerator("aurora").defaultColors;

const AURORA: BackgroundPreset = {
  background: {
    colors: auroraColors,
    generator: "aurora",
    kind: "mesh",
    lockedColors: auroraColors.map(() => false),
    seed: 1,
    warpPercent: 8,
  },
  id: "mesh-aurora",
  name: "Aurora",
};

const BLUE: BackgroundPreset = {
  background: { color: "#2563EB", kind: "solid" },
  id: "solid-blue",
  name: "Blue",
};

const PRESETS = [AURORA, BLUE];

const PURPLE: Background = { color: "#7A4FD1", kind: "solid" };

/** The picker driven as the hook drives it: each press changes the memory
 * and the canvas, and every read takes in what the canvas now shows. */
function picker(value: Background, saved: BackgroundPreset[] = []) {
  let memory = NO_CHOICE_MEMORY;
  let canvas = value;
  let kept = saved;
  const read = () => {
    const selection = backgroundSelection({
      memory,
      presets: PRESETS,
      savedPresets: kept,
      value: canvas,
    });
    memory = selection.memory;
    return selection;
  };
  return {
    get canvas() {
      return canvas;
    },
    /** A change from the colours line. */
    edit: (next: Background) => {
      read();
      memory = memoryAfter.edit(memory);
      canvas = next;
      return read();
    },
    /** A right click on Custom; the kept tile arrives when `arrives`. */
    keep: ({ arrives = true } = {}) => {
      const { canKeep, custom } = read();
      if (!canKeep || !custom) return { arrive: () => read() };
      memory = memoryAfter.keep();
      const arrive = () => {
        kept = [...kept, { background: custom, id: "kept", name: "Kept" }];
        return read();
      };
      if (arrives) arrive();
      return { arrive };
    },
    pick: (preset: BackgroundPreset) => {
      read();
      memory = memoryAfter.tile(memory);
      canvas = presetBackground(preset, kept.includes(preset));
      return read();
    },
    pressCustom: () => {
      const selection = read();
      memory = memoryAfter.openCustom(memory);
      canvas = customBackground(selection, canvas) ?? canvas;
      return read();
    },
    read,
    remove: (id: string) => {
      kept = kept.filter((preset) => preset.id !== id);
      return read();
    },
    get saved() {
      return kept;
    },
  };
}

describe("background choice", () => {
  it("chooses a built-in mesh's tile and leaves Custom empty", () => {
    const frame = picker(BLUE.background);
    const picked = frame.pick(AURORA);
    expect(picked.selectedId).toBe(AURORA.id);
    expect(picked.custom).toBeNull();
    expect(picked.showEditor).toBe(true);
  });

  it("moves to Custom once a built-in mesh's colours change", () => {
    const frame = picker(BLUE.background);
    frame.pick(AURORA);
    const mesh = frame.canvas as Extract<Background, { kind: "mesh" }>;
    const edited = frame.edit({ ...mesh, colors: ["#111111", "#222222"] });
    expect(edited.selectedId).toBe("custom");
    expect(edited.custom).toBe(frame.canvas);
    // Its own tile puts its own colours back.
    const again = frame.pick(AURORA);
    expect(again.selectedId).toBe(AURORA.id);
    expect(frame.canvas).toMatchObject({ colors: auroraColors });
  });

  it("keeps a colour of your own while other tiles are tried", () => {
    const frame = picker(PURPLE);
    expect(frame.read().selectedId).toBe("custom");
    const tried = frame.pick(AURORA);
    expect(tried.selectedId).toBe(AURORA.id);
    expect(tried.custom).toEqual(PURPLE);
    const back = frame.pressCustom();
    expect(frame.canvas).toEqual(PURPLE);
    expect(back.selectedId).toBe("custom");
  });

  it("starts a colour from a mesh's first when there is none of your own", () => {
    const frame = picker(BLUE.background);
    frame.pick(AURORA);
    const started = frame.pressCustom();
    expect(frame.canvas).toEqual({ color: auroraColors[0], kind: "solid" });
    expect(started.selectedId).toBe("custom");
  });

  it("holds Custom over a colour that matches a built-in tile", () => {
    const frame = picker(BLUE.background);
    const held = frame.pressCustom();
    expect(frame.canvas).toEqual(BLUE.background);
    expect(held.selectedId).toBe("custom");
    // A built-in tile already is that colour, so it is not kept again.
    expect(held.canKeep).toBe(false);
  });

  it("chooses the kept tile and empties Custom on keeping", () => {
    const frame = picker(PURPLE);
    frame.keep();
    const after = frame.read();
    expect(after.selectedId).toBe("kept");
    expect(after.custom).toBeNull();
    expect(frame.saved.map((preset) => preset.background)).toEqual([PURPLE]);
  });

  it("empties Custom when the kept tile arrives late", () => {
    const frame = picker(PURPLE);
    const { arrive } = frame.keep({ arrives: false });
    frame.read();
    const arrived = arrive();
    expect(arrived.selectedId).toBe("kept");
    expect(arrived.custom).toBeNull();
  });

  it("returns a removed background still on the canvas to Custom", () => {
    const frame = picker(PURPLE);
    frame.keep();
    const removed = frame.remove("kept");
    expect(removed.selectedId).toBe("custom");
    expect(removed.custom).toEqual(PURPLE);
  });

  it("never treats a picture as one of your own", () => {
    const picture: Background = { kind: "image", path: "/a.png" };
    const frame = picker(picture, [
      { background: picture, id: "picture", name: "a" },
    ]);
    const shown = frame.read();
    expect(shown.selectedId).toBe("picture");
    expect(shown.showEditor).toBe(false);
    expect(shown.canKeep).toBe(false);
  });
});
