// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { REGIONS } from "./glide-island-preview";
import { glideIslandMeta } from "./glide-island-story-meta";

import type { GlideIslandStage } from "./glide-island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...glideIslandMeta,
  parameters: {
    ...glideIslandMeta.parameters,
    stage: { height: 240, width: 640 },
  },
  title: "Features/Glide/Island/Regions",
} satisfies Meta<typeof GlideIslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Halves: Story = { args: { regions: REGIONS.halves } };

export const Thirds: Story = { args: { regions: REGIONS.thirds } };

export const TwoThirds: Story = { args: { regions: REGIONS.twoThirds } };

export const Quarters: Story = { args: { regions: REGIONS.quarters } };

export const Rows: Story = { args: { regions: REGIONS.rows } };

export const FullScreen: Story = { args: { regions: REGIONS.fullScreen } };
