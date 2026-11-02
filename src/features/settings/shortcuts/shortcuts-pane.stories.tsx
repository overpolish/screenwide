// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { userEvent, within } from "storybook/test";

import { t } from "../../../i18n/i18n";
import { SettingsWindow } from "../settings-window";
import { settingsWindowMeta } from "../settings-window-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...settingsWindowMeta,
  args: { initialSection: "hotkeys" },
  title: "Features/Settings/Shortcuts/Pane",
} satisfies Meta<typeof SettingsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Windows: Story = { parameters: { platform: "windows" } };

export const RightToLeft: Story = { globals: { direction: "rtl" } };

// Start or stop recording chosen in the list, then R clicked on the map
// with no modifier latched: Show ruler has the first combination, so it
// takes the next free one.
export const KeyFromMap: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const row = await canvas.findByText(t("settings-shortcut-start-stop"));
    await userEvent.click(row);
    const map = canvas.getByRole("group", {
      name: t("settings-shortcuts-map"),
    });
    // Each modifier is on the keyboard twice; both let go together.
    let latched = within(map).queryAllByRole("button", { pressed: true });
    while (latched.length > 0) {
      await userEvent.click(latched[0]);
      latched = within(map).queryAllByRole("button", { pressed: true });
    }
    const [r] = within(map).getAllByRole("button", { name: /^R\b/ });
    await userEvent.click(r);
  },
};
