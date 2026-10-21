// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { Slider } from "./slider";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    showTicks: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    "aria-label": "Volume",
    className: "w-64",
    defaultValue: 40,
    maxValue: 100,
    minValue: 0,
    onChange: fn(),
    onChangeEnd: fn(),
  },
  component: Slider,
  parameters: {
    controls: {
      exclude: ["aria-label", "className", "onChange", "onChangeEnd"],
    },
    layout: "centered",
  },
  title: "Primitives/Slider",
} satisfies Meta<typeof Slider>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Stepped: Story = {
  args: {
    defaultValue: 4,
    maxValue: 10,
    showTicks: true,
    step: 1,
  },
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      <StoryGridRow label="continuous">
        <Slider {...args} isDisabled />
      </StoryGridRow>
      <StoryGridRow label="stepped">
        <Slider
          {...args}
          defaultValue={4}
          isDisabled
          maxValue={10}
          showTicks
          step={1}
        />
      </StoryGridRow>
    </StoryGrid>
  ),
};
