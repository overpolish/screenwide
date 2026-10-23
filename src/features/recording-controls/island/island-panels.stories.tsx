// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { islandMeta } from "./island-story-meta";

import type { IslandStage } from "./island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...islandMeta,
  title: "Features/Recording Island/Panels",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Microphone: Story = { args: { initialPanel: "microphone" } };

export const Camera: Story = { args: { initialPanel: "camera" } };

export const CameraFourByThree: Story = {
  args: { camera: { selectedIds: ["opal"] }, initialPanel: "camera" },
};

export const CameraSquare: Story = {
  args: { camera: { selectedIds: ["obs"] }, initialPanel: "camera" },
};

export const CameraPortrait: Story = {
  args: { camera: { selectedIds: ["continuity"] }, initialPanel: "camera" },
};

export const CameraFlipped: Story = {
  args: { initialPanel: "camera", isCameraFlipped: true },
};

export const SystemAudioApps: Story = {
  args: {
    initialPanel: "systemAudio",
    systemAudio: { selectedIds: ["safari", "zoom"] },
  },
};

export const Screenshot: Story = { args: { initialPanel: "screenshot" } };

export const RecordOptions: Story = { args: { initialPanel: "record" } };

export const ReplayStarting: Story = {
  args: {
    initialPanel: "record",
    replay: { status: "starting" },
  },
};
