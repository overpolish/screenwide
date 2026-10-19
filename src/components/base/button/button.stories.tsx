// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Circle, ClipboardCopy, Upload } from "lucide-react";
import { ComponentProps } from "react";

import { StoryGrid, StoryGridRow } from "../../../storybook/story-grid";

import { Button } from "./button";

import type { Meta, StoryObj } from "@storybook/react-vite";

type ButtonProps = ComponentProps<typeof Button>;

const variants: NonNullable<ButtonProps["variant"]>[] = ["solid", "ghost"];
const colors: NonNullable<ButtonProps["color"]>[] = ["neutral", "primary"];
const sizes: NonNullable<ButtonProps["size"]>[] = ["regular", "capture"];

// Colours down, variants across: each colour changes how a variant looks.
function VariantGrid(args: ButtonProps) {
  return (
    <StoryGrid columns={variants}>
      {colors.map((color) => (
        <StoryGridRow key={color} label={color}>
          {variants.map((variant) => (
            <Button key={variant} {...args} color={color} variant={variant} />
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

const meta = {
  argTypes: {
    children: { control: "text" },
    color: {
      control: "inline-radio",
      options: colors,
      table: { defaultValue: { summary: "neutral" } },
    },
    isDisabled: { control: "boolean" },
    size: {
      control: "inline-radio",
      options: sizes,
      table: { defaultValue: { summary: "regular" } },
    },
    variant: {
      control: "inline-radio",
      options: variants,
      table: { defaultValue: { summary: "solid" } },
    },
  },
  args: {
    children: "Sign out",
    color: "neutral",
    isDisabled: false,
    size: "regular",
    variant: "solid",
  },
  component: Button,
  parameters: {
    controls: { exclude: ["ref", "className", "slot"] },
    layout: "centered",
  },
  title: "Primitives/Button",
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => <VariantGrid {...args} />,
};

export const Disabled: Story = {
  args: { isDisabled: true },
  parameters: { controls: { disable: true } },
  render: (args) => <VariantGrid {...args} />,
};

export const Sizes: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={3}>
      {sizes.map((size) => (
        <StoryGridRow key={size} label={size}>
          <Button {...args} size={size} />
          <Button {...args} color="primary" size={size} />
          {/* Icon only: the capture size is exactly 48 wide with a lone
              28px glyph. */}
          <Button {...args} aria-label="Record" color="primary" size={size}>
            <Circle />
          </Button>
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

export const WithIcon: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex items-center gap-control">
      <Button {...args} variant="ghost">
        <ClipboardCopy />
        Copy
      </Button>
      <Button {...args} color="primary">
        <Upload />
        Export
      </Button>
    </div>
  ),
};
