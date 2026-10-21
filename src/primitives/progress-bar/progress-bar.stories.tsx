// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { ProgressBar } from "./progress-bar";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isIndeterminate: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    value: { control: { max: 100, min: 0, type: "range" } },
  },
  args: {
    "aria-label": "Progress",
    className: "w-64",
    isIndeterminate: false,
    value: 62,
  },
  component: ProgressBar,
  parameters: {
    controls: { include: ["isIndeterminate", "value"] },
    layout: "centered",
  },
  title: "Primitives/Progress Bar",
} satisfies Meta<typeof ProgressBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      <StoryGridRow label="determinate">
        <ProgressBar {...args} isIndeterminate={false} />
      </StoryGridRow>
      <StoryGridRow label="indeterminate">
        <ProgressBar {...args} isIndeterminate />
      </StoryGridRow>
    </StoryGrid>
  ),
};
