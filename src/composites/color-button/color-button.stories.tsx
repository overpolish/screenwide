// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { ColorButton } from "./color-button";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    label: { control: "text" },
    value: { control: "color" },
  },
  args: {
    isDisabled: false,
    label: "Text",
    onPress: fn(),
    value: "#FFD60A",
  },
  component: ColorButton,
  parameters: {
    // Default is the labelled form; the swatch-only form, named by
    // `aria-label` instead, is shown under States.
    controls: { exclude: ["aria-expanded", "aria-label", "onPress", "ref"] },
    layout: "centered",
  },
  title: "Composites/Color Button",
} satisfies Meta<typeof ColorButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// A strong colour, white and a translucent one, which keep their outline by
// the hairline edge, in both forms.
const COLORS = [
  { label: "colour", value: "#FFD60A" },
  { label: "white", value: "#FFFFFF" },
  { label: "translucent", value: "#0A84FF40" },
];

function FormGrid({
  isDisabled,
  onPress,
}: {
  onPress: ComponentProps<typeof ColorButton>["onPress"];
  isDisabled?: boolean;
}) {
  return (
    <StoryGrid columns={["labelled", "swatch only"]}>
      {COLORS.map(({ label, value }) => (
        <StoryGridRow key={label} label={label}>
          <ColorButton
            isDisabled={isDisabled}
            label="Text"
            onPress={onPress}
            value={value}
          />
          <ColorButton
            aria-label="Text"
            isDisabled={isDisabled}
            onPress={onPress}
            value={value}
          />
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress }) => <FormGrid onPress={onPress} />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress }) => <FormGrid isDisabled onPress={onPress} />,
};
