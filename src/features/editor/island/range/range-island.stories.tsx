// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";

import { rangeIsland, type RangeIslandProps } from "./range-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function RangeIsland(props: RangeIslandProps) {
  return <EditorIslandStage content={rangeIsland(props)} />;
}

const meta = {
  args: { id: "range", onCut: fn() },
  component: RangeIsland,
  decorators: [featureStage({ height: 160, width: 480 })],
  parameters: {
    controls: { disable: true },
    layout: "fullscreen",
  },
  title: "Features/Editor/Island/Range",
} satisfies Meta<typeof RangeIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
