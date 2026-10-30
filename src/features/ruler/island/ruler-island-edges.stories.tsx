// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PREVIEW_RULER_STATE as state } from "./ruler-island-preview";
import { rulerIslandMeta } from "./ruler-island-story-meta";

import type { RulerIsland } from "./ruler-island";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...rulerIslandMeta,
  args: { ...rulerIslandMeta.args, initialShowsEdges: true },
  title: "Features/Ruler/Island/Edges",
} satisfies Meta<typeof RulerIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Balanced: Story = {};

export const SubtleEdges: Story = {
  args: { state: { ...state, tolerance: "subtleEdges" } },
};
