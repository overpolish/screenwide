// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { CircularProgress } from "./circular-progress";

const meta = {
  argTypes: {
    isIndeterminate: { control: "boolean" },
    size: { control: "inline-radio", options: ["small", "regular"] },
    value: { control: { max: 100, min: 0, type: "range" } },
  },
  args: {
    "aria-label": "Example progress",
    isIndeterminate: true,
    size: "regular",
    value: 62,
  },
  component: CircularProgress,
  parameters: {
    controls: { include: ["isIndeterminate", "size", "value"] },
    layout: "centered",
  },
  title: "Primitives/Circular Progress",
} satisfies Meta<typeof CircularProgress>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Indeterminate: Story = {};

export const Determinate: Story = {
  args: { isIndeterminate: false },
};

export const Sizes: Story = {
  parameters: {
    controls: { disable: true },
    layout: "padded",
  },
  render: () => (
    <div className="flex items-center gap-section">
      <CircularProgress
        aria-label="Small indeterminate progress"
        isIndeterminate
        size="small"
      />
      <CircularProgress
        aria-label="Regular indeterminate progress"
        isIndeterminate
        size="regular"
      />
      <CircularProgress
        aria-label="Small determinate progress"
        size="small"
        value={62}
      />
      <CircularProgress
        aria-label="Regular determinate progress"
        size="regular"
        value={62}
      />
    </div>
  ),
};
