// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SettingsWindow } from "./settings-window";
import { settingsWindowMeta } from "./settings-window-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...settingsWindowMeta,
  args: { initialSection: "transcription" },
  title: "Legacy/Settings/Models",
} satisfies Meta<typeof SettingsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Downloaded: Story = {};

export const NotDownloaded: Story = {
  parameters: { modelSeed: { progress: null, status: "available" } },
};

export const Downloading: Story = {
  parameters: { modelSeed: { progress: 0.42, status: "downloading" } },
};
