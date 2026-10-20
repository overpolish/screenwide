// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { ColorButton } from "./color-button";

import type { Meta, StoryObj } from "@storybook/react-vite";

type ColorButtonProps = ComponentProps<typeof ColorButton>;

// Both presentations, a row each, with the given state.
function PresentationGrid({
  isDisabled,
  isLocked,
  onPress,
  value,
}: Pick<ColorButtonProps, "isDisabled" | "isLocked" | "onPress" | "value">) {
  const state = { isDisabled, isLocked, onPress, value };
  return (
    <StoryGrid>
      <StoryGridRow label="labelled">
        <ColorButton {...state} label="Text" />
      </StoryGridRow>
      <StoryGridRow label="swatch only">
        <ColorButton {...state} aria-label="Text" />
      </StoryGridRow>
    </StoryGrid>
  );
}

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    isLocked: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    label: { control: "text" },
    value: { control: "color" },
  },
  args: {
    isDisabled: false,
    isLocked: false,
    label: "Text",
    onPress: fn(),
    value: "#FFD60A",
  },
  component: ColorButton,
  parameters: {
    // Default is the labelled form; the swatch-only form, named by
    // `aria-label` instead, is shown under Presentations. `aria-expanded`
    // is only announced, so there is nothing to see in changing it.
    controls: { exclude: ["aria-expanded", "aria-label", "onPress", "ref"] },
    layout: "centered",
  },
  title: "Composites/Color Button",
} satisfies Meta<typeof ColorButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Presentations: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress, value }) => (
    <PresentationGrid onPress={onPress} value={value} />
  ),
};

export const Locked: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress, value }) => (
    <PresentationGrid isLocked onPress={onPress} value={value} />
  ),
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress, value }) => (
    <PresentationGrid isDisabled onPress={onPress} value={value} />
  ),
};
