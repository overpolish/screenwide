// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { Background, BackgroundPreset, presetName } from "./background";
import { BackgroundPicker } from "./background-picker";
import { BUILT_IN_BACKGROUND_PRESETS } from "./background-presets";

import type { Meta, StoryObj } from "@storybook/react-vite";

const savedPresets: BackgroundPreset[] = [
  {
    background: { color: "#0B3D2E", kind: "solid" },
    id: "saved-forest",
    name: "Forest",
  },
  {
    background: {
      colors: ["#1A1A2E", "#16213E", "#0F3460", "#E94560"],
      generator: "mesh",
      kind: "mesh",
      lockedColors: [false, false, false, false],
      points: [
        { radiusX: 80, radiusY: 60, rotation: 10, x: 20, y: 20 },
        { radiusX: 70, radiusY: 70, rotation: -20, x: 80, y: 40 },
        { radiusX: 90, radiusY: 55, rotation: 45, x: 50, y: 95 },
      ],
      seed: 991,
      warpPercent: 10,
    },
    id: "saved-nebula",
    name: "Nebula",
  },
];

/** The picker owns nothing: the story holds the chosen background the way the
 * editor's canvas does, and reports what a save or a removal asked for. */
function PickerHarness({ initial }: { initial: Background }) {
  const [value, setValue] = useState(initial);
  const [saved, setSaved] = useState(savedPresets);

  return (
    <div className="w-64">
      <BackgroundPicker
        onChange={setValue}
        onPickImage={() => {
          // A story cannot open a native picker, so it answers with the
          // picture it ships with.
          fn()("pick image");
          return Promise.resolve("/screenwide/story-background.jpg");
        }}
        onPresetMenu={(id) => {
          // The app answers a right click with the pop-up panel, which a
          // story has no window for, so it removes the preset outright.
          fn()("preset menu", id);
          setSaved((current) => current.filter((preset) => preset.id !== id));
        }}
        onSavePreset={(background) => {
          const name = presetName(background);
          fn()("save preset", background);
          setSaved((current) => [
            ...current,
            { background, id: `saved-${String(current.length)}`, name },
          ]);
        }}
        presets={BUILT_IN_BACKGROUND_PRESETS}
        savedPresets={saved}
        value={value}
      />
    </div>
  );
}

const meta = {
  component: PickerHarness,
  parameters: { layout: "centered" },
  title: "Components/Background Picker",
} satisfies Meta<typeof PickerHarness>;

export default meta;
type Story = StoryObj<typeof meta>;

/* --------------------------------- Stories -------------------------------- */

/** One of the backgrounds the app ships with, chosen. The first nine tiles
 * are the nine mesh generators, each under the palette it ships with. */
export const Default: Story = {
  args: { initial: BUILT_IN_BACKGROUND_PRESETS[0].background },
};

/** A generator that is not the composition one: its tile is selected, and
 * its palette is the whole of what it is given. */
export const ProceduralGenerator: Story = {
  args: {
    initial:
      BUILT_IN_BACKGROUND_PRESETS.find((preset) => preset.id === "mesh-aurora")
        ?.background ?? BUILT_IN_BACKGROUND_PRESETS[0].background,
  },
};

/** A mesh that matches nothing on offer: the editor is open on it already,
 * ready to be tuned or saved under a name. */
export const Custom: Story = {
  args: {
    initial: {
      colors: ["#2A0A18", "#7B2D5E", "#F2A65A", "#FFE8C2"],
      generator: "mesh",
      kind: "mesh",
      lockedColors: [false, true, false, false],
      points: [
        { radiusX: 85, radiusY: 65, rotation: 24, x: 10, y: 18 },
        { radiusX: 60, radiusY: 80, rotation: -40, x: 92, y: 30 },
        { radiusX: 95, radiusY: 50, rotation: 70, x: 48, y: 98 },
      ],
      seed: 12_345,
      warpPercent: 11,
    },
  },
};

/** A picture of your own: the Image tile carries it rather than its glyph. */
export const Image: Story = {
  args: {
    initial: { kind: "image", path: "/screenwide/story-background.jpg" },
  },
};

/** Backgrounds saved from earlier canvases sit after the built-in ones, and
 * offer removal on a right click, through the app's own pop-up menu. */
export const WithSavedPresets: Story = {
  args: { initial: savedPresets[0].background },
};
