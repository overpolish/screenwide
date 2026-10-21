// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { SoftwareUpdateSetting } from "./settings-update-actions";
import { SettingsWindow } from "./settings-window";
import { settingsWindowMeta } from "./settings-window-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...settingsWindowMeta,
  args: { initialSection: "general" },
  title: "Features/Settings/General",
} satisfies Meta<typeof SettingsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const UpdateAvailable: Story = {
  args: {
    updateSetting: (
      <SoftwareUpdateSetting
        currentVersion="0.1.0"
        error={null}
        onPress={fn()}
        status="available"
        updateVersion={"1.0.0"}
      />
    ),
  },
};

export const UpdateChecking: Story = {
  args: {
    updateSetting: (
      <SoftwareUpdateSetting
        currentVersion="0.1.0"
        error={null}
        onPress={fn()}
        status="checking"
        updateVersion={null}
      />
    ),
  },
};

export const UpdateError: Story = {
  args: {
    updateSetting: (
      <SoftwareUpdateSetting
        currentVersion="0.1.0"
        error={"Could not reach the update server."}
        onPress={fn()}
        status="error"
        updateVersion={null}
      />
    ),
  },
};
