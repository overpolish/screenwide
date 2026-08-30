// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { DoorOpen, HandMetal } from "lucide-react";

import { Button } from "./button";

const sizes: React.ComponentProps<typeof Button>["size"][] = [
  "default",
  "compact",
] as const;
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
    size: {
      control: "inline-radio",
      options: sizes,
      table: { defaultValue: { summary: "default" } },
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
    size: "default",
    variant: "solid",
    isDisabled: false,
    /* eslint-enable sort-keys */
  },
};

export const Sizes: Story = {
  parameters: {
    controls: { disable: true },
  },
  render: (args) => (
    <div className="flex gap-2 items-center">
      {sizes.map((size) => (
        <Button key={size} size={size} {...args} />
      ))}
    </div>
  ),
};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex gap-2 items-center">
      {variants.map((variant) => (
        <Button color="primary" key={variant} variant={variant} {...args} />
      ))}
    </div>
  ),
};

export const Colors: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex gap-2 items-center">
      {colors.map((color) => (
        <Button key={color} {...args} color={color} />
      ))}
    </div>
  ),
};

export const WithElements: Story = {
  args: {
    children: (
      <>
        <DoorOpen />
        Sign out
        <HandMetal />
      </>
    ),
  },
  parameters: { controls: { disable: true } },
};
