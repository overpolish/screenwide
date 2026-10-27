// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { DoorOpen } from "lucide-react";
import { ComponentProps } from "react";
import { fn } from "storybook/test";

import { onIsland } from "../../storybook/on-island";
import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { IconButton } from "./icon-button";

import type { Meta, StoryObj } from "@storybook/react-vite";

type IconButtonProps = ComponentProps<typeof IconButton>;

// Button's looks: a solid one in either colour, or a ghost.
const looks = [
  { label: "neutral", look: { variant: "solid" } },
  { label: "primary", look: { color: "primary", variant: "solid" } },
  { label: "ghost", look: { variant: "ghost" } },
] as const;

function LookGrid({
  isDisabled,
  onPress,
}: Pick<IconButtonProps, "isDisabled" | "onPress">) {
  return (
    <StoryGrid>
      {looks.map(({ label, look }) => (
        <StoryGridRow key={label} label={label}>
          <IconButton
            {...look}
            aria-label="Sign out"
            isDisabled={isDisabled}
            onPress={onPress}
          >
            <DoorOpen />
          </IconButton>
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

const meta = {
  argTypes: {
    color: {
      control: "inline-radio",
      options: ["neutral", "primary"],
      table: { defaultValue: { summary: "neutral" } },
    },
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    variant: {
      control: "inline-radio",
      options: ["solid", "ghost"],
      table: { defaultValue: { summary: "solid" } },
    },
  },
  args: {
    "aria-label": "Sign out",
    children: <DoorOpen />,
    color: "neutral",
    isDisabled: false,
    onPress: fn(),
    variant: "solid",
  },
  component: IconButton,
  parameters: {
    controls: { include: ["aria-label", "color", "isDisabled", "variant"] },
    layout: "centered",
  },
  title: "Primitives/Icon Button",
} satisfies Meta<typeof IconButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress }) => <LookGrid onPress={onPress} />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress }) => <LookGrid isDisabled onPress={onPress} />,
};

export const Island: Story = { ...Variants, decorators: [onIsland()] };
