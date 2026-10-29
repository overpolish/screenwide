// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { presetName } from "../../tool-panels/frame/background/background";
import { builtInBackgroundPresets } from "../../tool-panels/frame/background/background-presets";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { frameIsland, type FrameIslandProps } from "./frame-island";

import type {
  Background,
  BackgroundPreset,
} from "../../tool-panels/frame/background/background";
import type { Meta, StoryObj } from "@storybook/react-vite";

/** A mesh kept from before, as a saved tile. */
const NEBULA: Background = {
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
};

const SAVED: BackgroundPreset[] = [
  { background: NEBULA, id: "saved-nebula", name: "Nebula" },
];

function FrameIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & FrameIslandProps) {
  return <EditorIslandStage content={frameIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    background: { color: "#2563EB", kind: "solid" },
    height: 1080,
    isShown: true,
    onBackgroundChange: fn(),
    onOriginalSize: fn(),
    // A story cannot open a native picker, so it answers with the picture
    // it ships with.
    onPickImage: fn(() => Promise.resolve("/screenwide/story-background.jpg")),
    onRadiusChange: fn(),
    onRemovePreset: fn(),
    onSavePreset: fn(),
    onSizeChange: fn(),
    presets: [],
    savedPresets: SAVED,
    width: 1920,
  },
  component: FrameIsland,
  decorators: [featureStage({ height: 420, width: 760 })],
  // Every choice changes the story's frame, as it would the picture's; the
  // built-in tiles are named in the reader's language, so they are made
  // while rendering.
  render: function Render(args) {
    const [, updateArgs] = useArgs<FrameIslandProps>();
    return (
      <FrameIsland
        {...args}
        onBackgroundChange={(background) => {
          args.onBackgroundChange(background);
          updateArgs({ background });
        }}
        onOriginalSize={() => {
          args.onOriginalSize();
          updateArgs({ height: 1080, width: 1920 });
        }}
        onRadiusChange={(radius) => {
          args.onRadiusChange?.(radius);
          updateArgs({ radius });
        }}
        onRemovePreset={(id) => {
          args.onRemovePreset(id);
          updateArgs({
            savedPresets: args.savedPresets.filter((kept) => kept.id !== id),
          });
        }}
        onSavePreset={(background) => {
          args.onSavePreset(background);
          updateArgs({
            savedPresets: [
              ...args.savedPresets,
              {
                background,
                // Unique past removals, as the tiles are keyed by it.
                id: `saved-${Date.now().toString(36)}`,
                name: presetName(background),
              },
            ],
          });
        }}
        onSizeChange={(size) => {
          args.onSizeChange(size);
          updateArgs(size);
        }}
        presets={builtInBackgroundPresets()}
      />
    );
  },
  title: "Features/Editor/Island/Frame",
} satisfies Meta<typeof FrameIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

// A video's corners are never rounded, so none are offered.
export const Recording: Story = {};

export const Mesh: Story = { args: { background: NEBULA } };

// A colour of your own, matching no tile: Custom is chosen, and its colour
// line shows under the tiles.
export const Custom: Story = {
  args: { background: { color: "#7A4FD1", kind: "solid" } },
};

// A screenshot's frame, the one place a picture's corners are rounded.
export const Screenshot: Story = { args: { radius: 4 } };
