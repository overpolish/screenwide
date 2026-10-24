// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { annotateIslandMeta } from "./annotate-island-story-meta";
import { PREVIEW_ANNOTATE_SETTINGS as settings } from "./annotate-preview";

import type { AnnotateIsland } from "./annotate-island";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...annotateIslandMeta,
  title: "Features/Annotate Island/Tools",
} satisfies Meta<typeof AnnotateIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Arrow: Story = {};

export const Counter: Story = {
  args: { settings: { ...settings, defaultShape: "counter" } },
};

export const Highlight: Story = {
  args: { settings: { ...settings, defaultShape: "highlight" } },
};

export const Shape: Story = {
  args: { settings: { ...settings, defaultShape: "shape" } },
};

export const Focus: Story = {
  args: { settings: { ...settings, defaultShape: "focus" } },
};

export const Draw: Story = {
  args: { settings: { ...settings, defaultShape: "draw" } },
};
