// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { SettingsWindow } from "./settings-window";
import { settingsWindowMeta } from "./settings-window-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A key that ends in a tool's name (`DEMO-GLIDE`) adds that tool; one with
// "refund" in it was refunded, "offline" finds no server, anything else is
// not recognised.
const meta = {
  ...settingsWindowMeta,
  args: { initialSection: "licences" },
  title: "Features/Settings/Licences",
} satisfies Meta<typeof SettingsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const InTrial: Story = {};

export const BeforeFirstUse: Story = {
  parameters: { licencesSeed: "unused" },
};

export const TrialEnded: Story = {
  parameters: { licencesSeed: "expired" },
};

export const Licensed: Story = {
  parameters: { licencesSeed: "licensed" },
};
