// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { islandMeta } from "./island-story-meta";

import type { IslandStage } from "./island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...islandMeta,
  title: "Features/Recording Island/Recording",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Countdown: Story = { args: { initialPhase: "countdown" } };

export const Starting: Story = { args: { initialPhase: "starting" } };

export const Recording: Story = { args: { initialPhase: "recording" } };

export const ConfidenceChecks: Story = {
  args: { hasConfidenceChecks: true, initialPhase: "recording" },
};

export const MomentInProgress: Story = {
  args: {
    initialPhase: "recording",
    moment: {
      color: "#f5a524",
      key: 1,
      name: "Highlight",
      noMicrophone: false,
      noteMs: 3_000,
    },
  },
};

export const RollingIntoHours: Story = {
  args: { initialElapsed: 3_595, initialPhase: "recording" },
};

export const SizeEstimate: Story = {
  args: { initialPhase: "recording", showsSize: true },
};

export const Paused: Story = { args: { initialPhase: "paused" } };

export const Finishing: Story = { args: { initialPhase: "finishing" } };
