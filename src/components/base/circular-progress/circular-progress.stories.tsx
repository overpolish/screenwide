// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { CircularProgress } from "./circular-progress";

const meta = {
  args: {
    "aria-label": "Example progress",
    isIndeterminate: true,
  },
  component: CircularProgress,
  parameters: {
    controls: { exclude: ["aria-label", "renderLabel"] },
    layout: "centered",
  },
  title: "Primitives/Circular Progress",
} satisfies Meta<typeof CircularProgress>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Indeterminate: Story = {};

export const Sizes: Story = {
  render: () => (
    <div className="flex items-center gap-section">
      <CircularProgress
        aria-label="Compact progress"
        isIndeterminate
        size="compact"
      />
      <CircularProgress aria-label="Default progress" isIndeterminate />
      <CircularProgress
        aria-label="Large progress"
        isIndeterminate
        size="large"
      />
    </div>
  ),
};

export const Determinate: Story = {
  args: {
    isIndeterminate: false,
    size: "large",
    value: 62,
  },
};
