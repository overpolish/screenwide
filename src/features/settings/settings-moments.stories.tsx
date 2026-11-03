// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { userEvent, within } from "storybook/test";

import { SettingsWindow } from "./settings-window";
import { settingsWindowMeta } from "./settings-window-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...settingsWindowMeta,
  args: { initialSection: "moments" },
  title: "Features/Settings/Moments",
} satisfies Meta<typeof SettingsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// The first moment's swatch pressed: its colour grid open under its row.
export const ColourPickerOpen: Story = {
  play: async ({ canvasElement }) => {
    const [swatch] = await within(canvasElement).findAllByRole("button", {
      expanded: false,
    });
    await userEvent.click(swatch);
  },
};
