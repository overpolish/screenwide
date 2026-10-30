// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { islandMeta } from "./island-story-meta";

import type { IslandStage } from "./island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...islandMeta,
  title: "Features/Recording Controls/Island/Resting",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const InputsOff: Story = {
  args: {
    camera: { isOn: false },
    microphone: { isOn: false },
    systemAudio: { isOn: false },
  },
};
