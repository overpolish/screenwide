// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { Crop, DoorOpen, Pause, Play } from "lucide-react";

import { Text } from "../text/text";

import { IconButton, IconToggleButton } from "./icon-button";

const colors: React.ComponentProps<typeof IconButton>["color"][] = [
  "neutral",
  "primary",
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

export const Colors: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex items-center gap-section">
      {colors.map((color) => (
        <IconButton key={color} {...args} color={color} />
      ))}
    </div>
  ),
};

export const States: Story = {
  // Padded rather than centered: centering lands the row on a fractional
  // pixel, which makes the bezel and glyph snap differently.
  parameters: { controls: { disable: true }, layout: "padded" },
  render: (args) => (
    <div className="flex flex-col gap-section">
      {colors.map((color) => (
        <div className="flex items-center gap-section" key={color}>
          <Text className="w-16" variant="footnote">
            {color}
          </Text>
          <IconButton {...args} color={color} />
          <IconButton {...args} color={color} isDisabled />
        </div>
      ))}
      <div className="flex items-center gap-section">
        <Text className="w-16" variant="footnote">
          toggle
        </Text>
        <IconToggleButton aria-label="Crop (off)" defaultSelected={false}>
          <Crop />
        </IconToggleButton>
        <IconToggleButton aria-label="Crop (on)" defaultSelected>
          <Crop />
        </IconToggleButton>
        <IconToggleButton
          aria-label="Crop (disabled)"
          defaultSelected
          isDisabled
        >
          <Crop />
        </IconToggleButton>
      </div>
    </div>
  ),
};

export const Toggle: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div className="flex items-center gap-section">
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

export const SingleIconToggle: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <IconToggleButton aria-label="Crop" defaultSelected>
      <Crop />
    </IconToggleButton>
  ),
};

export const DisabledSingleIconToggle: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <IconToggleButton aria-label="Crop" defaultSelected isDisabled>
      <Crop />
    </IconToggleButton>
  ),
};
