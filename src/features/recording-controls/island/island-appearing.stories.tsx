// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { islandMeta } from "./island-story-meta";

import type { IslandStage } from "./island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...islandMeta,
  args: { ...islandMeta.args, appears: true },
  title: "Features/Recording Controls/Island/Appearing",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Appear: Story = {};
