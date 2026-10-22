// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { islandMeta } from "./island-story-meta";

import type { IslandStage } from "./island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...islandMeta,
  title: "Features/Recording Controls/Island/Warnings",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const MissingMicrophone: Story = {
  args: { microphoneProblem: "missing" },
};

export const NoScreenMissingMicrophone: Story = {
  args: {
    camera: { isOn: false },
    initialMode: "none",
    microphoneProblem: "missing",
    systemAudio: { isOn: false },
  },
};

export const CameraLocked: Story = { args: { camera: { isLocked: true } } };

export const ScreenLocked: Story = { args: { isScreenLocked: true } };

export const SilentMicrophone: Story = {
  args: { initialPhase: "recording", microphoneProblem: "silent" },
};
