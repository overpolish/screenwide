// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Flag } from "lucide-react";

import { onIsland } from "../../../storybook/on-island";

import { IslandRowTitle } from "./island-row-title";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: { title: "Flagged moment" },
  component: IslandRowTitle,
  decorators: [onIsland()],
  parameters: {
    controls: { include: ["title", "note"] },
    layout: "centered",
  },
  title: "Primitives/Island/Row Title",
} satisfies Meta<typeof IslandRowTitle>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithNote: Story = { args: { note: "1:24, 6 seconds" } };

export const WithGlyph: Story = {
  args: { note: "1:24, 6 seconds" },
  render: (args) => <IslandRowTitle {...args} glyph={<Flag />} />,
};
