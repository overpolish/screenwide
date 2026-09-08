// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { ProgressBar } from "./progress-bar";

const meta = {
  argTypes: {
    isIndeterminate: { control: "boolean" },
    value: { control: { max: 100, min: 0, type: "range" } },
  },
  args: {
    "aria-label": "Example progress",
    isIndeterminate: false,
    value: 62,
  },
  component: ProgressBar,
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { include: ["isIndeterminate", "value"] },
    layout: "padded",
  },
  title: "Primitives/Progress Bar",
} satisfies Meta<typeof ProgressBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Determinate: Story = {};

/** Work that cannot report how far it has come sweeps instead. */
export const Indeterminate: Story = {
  args: { isIndeterminate: true },
};
