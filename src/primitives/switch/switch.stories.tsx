// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { Switch } from "./switch";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    isSelected: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    "aria-label": "Show cursor",
    isDisabled: false,
    isSelected: false,
    onChange: fn(),
  },
  component: Switch,
  parameters: {
    controls: { include: ["isDisabled", "isSelected"] },
    layout: "centered",
  },
  // Switching keeps the control in step.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <Switch
        {...args}
        onChange={(isSelected) => {
          args.onChange?.(isSelected);
          updateArgs({ isSelected });
        }}
      />
    );
  },
  title: "Primitives/Switch",
} satisfies Meta<typeof Switch>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      <StoryGridRow label="off">
        <Switch {...args} isSelected={false} />
      </StoryGridRow>
      <StoryGridRow label="on">
        <Switch {...args} isSelected />
      </StoryGridRow>
    </StoryGrid>
  ),
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      <StoryGridRow label="off">
        <Switch {...args} isDisabled isSelected={false} />
      </StoryGridRow>
      <StoryGridRow label="on">
        <Switch {...args} isDisabled isSelected />
      </StoryGridRow>
    </StoryGrid>
  ),
};

export const WithLabel: Story = {
  args: { "aria-label": undefined, children: "Show cursor" },
};
