// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Slider } from "./slider";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    "aria-label": "Value",
    defaultValue: 40,
    maxValue: 100,
    minValue: 0,
  },
  component: Slider,
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
  parameters: {
    layout: "centered",
  },
  title: "Primitives/Slider",
} satisfies Meta<typeof Slider>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const SetValue: Story = {
  args: {
    defaultValue: 75,
  },
};

export const Disabled: Story = {
  args: {
    isDisabled: true,
  },
};

export const ConstrainedWidth: Story = {
  decorators: [
    (Story) => (
      <div className="w-40">
        <Story />
      </div>
    ),
  ],
};
