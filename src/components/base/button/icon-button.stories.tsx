// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { DoorOpen, Pause, Play } from "lucide-react";

import { IconButton, IconToggleButton } from "./icon-button";

const sizes: React.ComponentProps<typeof IconButton>["size"][] = [
  "default",
  "compact",
] as const;
const colors: React.ComponentProps<typeof IconButton>["color"][] = [
  "neutral",
  "primary",
] as const;
const iconSizes: React.ComponentProps<typeof IconButton>["iconSize"][] = [
  undefined,
  "prominent",
] as const;

const meta = {
  argTypes: {
    color: { control: "inline-radio", options: colors },
  },
  args: {
    "aria-label": "Sign out",
    children: <DoorOpen />,
    color: "neutral",
    isDisabled: false,
    size: "default",
  },
  component: IconButton,
  parameters: {
    controls: { exclude: ["children", "className", "ref"] },
    layout: "centered",
  },
  title: "Primitives/Icon Button",
} satisfies Meta<typeof IconButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Sizes: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex items-center gap-2">
      {sizes.map((size) => (
        <IconButton key={size} {...args} size={size} />
      ))}
    </div>
  ),
};

export const Colors: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex items-center gap-2">
      {colors.map((color) => (
        <IconButton key={color} {...args} color={color} />
      ))}
    </div>
  ),
};

export const IconSizes: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex items-center gap-2">
      {iconSizes.map((iconSize) => (
        <IconButton key={iconSize ?? "default"} {...args} iconSize={iconSize} />
      ))}
    </div>
  ),
};

export const Toggle: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div className="flex items-center gap-2">
      <IconToggleButton
        aria-label="Play"
        defaultSelected={false}
        off={<Play />}
      >
        <Pause />
      </IconToggleButton>
      <IconToggleButton aria-label="Pause" defaultSelected off={<Play />}>
        <Pause />
      </IconToggleButton>
    </div>
  ),
};
