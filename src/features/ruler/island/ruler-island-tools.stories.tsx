// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PREVIEW_RULER_STATE as state } from "./ruler-island-preview";
import { rulerIslandMeta } from "./ruler-island-story-meta";

import type { RulerIsland } from "./ruler-island";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...rulerIslandMeta,
  title: "Features/Ruler/Island/Tools",
} satisfies Meta<typeof RulerIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Measure: Story = {};

export const VerticalGuide: Story = {
  args: { state: { ...state, tool: "verticalGuide" } },
};

export const HorizontalGuide: Story = {
  args: { state: { ...state, tool: "horizontalGuide" } },
};

export const Radius: Story = {
  args: { state: { ...state, tool: "radius" } },
};

export const NothingMeasured: Story = {
  args: { state: { ...state, canCopy: false, canUndo: false } },
};
