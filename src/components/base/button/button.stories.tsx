// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { DoorOpen } from "lucide-react";

import { Text } from "../text/text";

import { Button } from "./button";

const variants: React.ComponentProps<typeof Button>["variant"][] = [
  "solid",
  "ghost",
] as const;
const colors: React.ComponentProps<typeof Button>["color"][] = [
  "primary",
  "neutral",
];

const meta = {
  argTypes: {
    color: {
      control: "inline-radio",
      options: colors,
    },
    isDisabled: {
      control: "boolean",
    },
    variant: {
      control: "inline-radio",
      options: variants,
      table: { defaultValue: { summary: "solid" } },
    },
  },
  args: {
    children: "Default",
    color: "neutral",
  },
  component: Button,
  parameters: {
    controls: {
      exclude: ["ref", "className", "slot"],
    },
    layout: "centered",
  },
  title: "Primitives/Button",
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

/* --------------------------------- Stories -------------------------------- */
export const Default: Story = {
  args: {
    /* eslint-disable sort-keys */
    variant: "solid",
    isDisabled: false,
    /* eslint-enable sort-keys */
  },
};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex gap-section items-center">
      {variants.map((variant) => (
        <Button color="primary" key={variant} variant={variant} {...args} />
      ))}
    </div>
  ),
};

export const Colors: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex gap-section items-center">
      {colors.map((color) => (
        <Button key={color} {...args} color={color} />
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
        <div className="flex gap-section items-center" key={color}>
          <Text className="w-16" variant="footnote">
            {color}
          </Text>
          <Button {...args} color={color}>
            Rest
          </Button>
          <Button {...args} color={color} isDisabled>
            Disabled
          </Button>
          <Button {...args} color={color} variant="ghost">
            Ghost
          </Button>
          <Button {...args} color={color} isDisabled variant="ghost">
            Ghost disabled
          </Button>
        </div>
      ))}
    </div>
  ),
};

/** A symbol goes before the label. Trailing glyphs are reserved for a menu
 * or disclosure indicator drawn by the control itself. */
export const WithIcon: Story = {
  args: {
    children: (
      <>
        <DoorOpen />
        Sign out
      </>
    ),
  },
  parameters: { controls: { disable: true } },
};
