// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Switch } from "./switch";

import type { Meta, StoryObj } from "@storybook/react";

const meta = {
  args: { children: "Setting" },
  component: Switch,
  parameters: {
    layout: "centered",
  },
  title: "Primitives/Switch",
} satisfies Meta<typeof Switch>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Off: Story = {};

export const On: Story = {
  args: { defaultSelected: true },
};

export const WithoutVisibleLabel: Story = {
  args: {
    "aria-label": "Setting",
    children: undefined,
  },
};

export const Disabled: Story = {
  render: () => (
    <div className="gap-section flex flex-col items-start">
      <Switch isDisabled>Unavailable</Switch>
      <Switch isDisabled isSelected>
        Enabled but unavailable
      </Switch>
    </div>
  ),
};
