// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SettingsWindow } from "./settings-window";
import { settingsWindowMeta } from "./settings-window-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...settingsWindowMeta,
  args: { initialQuery: "recording" },
  title: "Legacy/Settings/Search",
} satisfies Meta<typeof SettingsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Results: Story = {};

export const NoResults: Story = { args: { initialQuery: "teleport" } };
