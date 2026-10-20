// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { CircularProgress } from "./circular-progress";

import type { Meta, StoryObj } from "@storybook/react-vite";

const sizes: NonNullable<ComponentProps<typeof CircularProgress>["size"]>[] = [
  "small",
  "regular",
];

const meta = {
  argTypes: {
    isIndeterminate: { control: "boolean" },
    size: {
      control: "inline-radio",
      options: sizes,
      table: { defaultValue: { summary: "regular" } },
    },
    value: { control: { max: 100, min: 0, type: "range" } },
  },
  args: {
    "aria-label": "Progress",
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

export const Default: Story = {};

export const Sizes: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["indeterminate", "determinate"]}>
      {sizes.map((size) => (
        <StoryGridRow key={size} label={size}>
          <CircularProgress {...args} isIndeterminate size={size} />
          <CircularProgress {...args} isIndeterminate={false} size={size} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
