// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { builtInBackgroundPresets } from "../../tools/frame/background/background-presets";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { gapIsland, type GapIslandProps } from "./gap-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function GapIsland({
  isShown,
  ...props
}: GapIslandProps & EditorIslandStoryProps) {
  return <EditorIslandStage content={gapIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    aspect: 16 / 9,
    background: { color: "#2563EB", kind: "solid" },
    hasOwnBackground: true,
    id: "gap",
    isShown: true,
    onBackgroundChange: fn(),
    onClearBackground: fn(),
    // A story cannot open a native picker, so it answers with the picture
    // it ships with.
    onPickImage: fn(() => Promise.resolve("/screenwide/story-background.jpg")),
    onRemove: fn(),
    onRemovePreset: fn(),
    onSavePreset: fn(),
    presets: builtInBackgroundPresets(),
    savedPresets: [],
  },
  component: GapIsland,
  decorators: [featureStage({ height: 420, width: 760 })],
  title: "Features/Editor/Island/Gap",
} satisfies Meta<typeof GapIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
