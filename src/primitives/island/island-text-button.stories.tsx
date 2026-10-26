// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { IslandTextButton } from "./island-controls";

import type { Meta, StoryObj } from "@storybook/react-vite";

const colors = ["neutral", "primary"] as const;

const meta = {
  argTypes: {
    color: {
      control: "inline-radio",
      options: colors,
      table: { defaultValue: { summary: "neutral" } },
    },
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    children: "Copy all",
    color: "neutral",
    isDisabled: false,
    onPress: fn(),
  },
  component: IslandTextButton,
  decorators: [
    // On the island's black, which is always drawn dark.
    (Story) => (
      <div className="dark rounded-panel bg-island p-inset text-fg">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { include: ["children", "color", "isDisabled"] },
    layout: "centered",
  },
  title: "Primitives/Island Text Button",
} satisfies Meta<typeof IslandTextButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Colors: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {colors.map((color) => (
        <StoryGridRow key={color} label={color}>
          <IslandTextButton {...args} color={color} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

export const Disabled: Story = {
  args: { isDisabled: true },
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {colors.map((color) => (
        <StoryGridRow key={color} label={color}>
          <IslandTextButton {...args} color={color} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
