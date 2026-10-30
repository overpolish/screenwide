// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SENT } from "./glide-island-preview";
import { glideIslandMeta } from "./glide-island-story-meta";

import type { GlideIslandStage } from "./glide-island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...glideIslandMeta,
  title: "Features/Glide/Island/Ending",
} satisfies Meta<typeof GlideIslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Sent: Story = { args: { state: SENT } };

export const Locked: Story = { args: { script: "locked" } };
