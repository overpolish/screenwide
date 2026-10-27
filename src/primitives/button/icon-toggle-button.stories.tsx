// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop, Pause, Play } from "lucide-react";
import { ComponentProps } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { onIsland } from "../../storybook/on-island";
import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { IconToggleButton } from "./icon-button";

import type { Meta, StoryObj } from "@storybook/react-vite";

type IconToggleButtonProps = ComponentProps<typeof IconToggleButton>;

const variants = ["ghost", "solid"] as const;
const states = [false, true];

// Each look as a row, off and on as its columns.
function VariantGrid({
  isDisabled,
  onChange,
}: Pick<IconToggleButtonProps, "isDisabled" | "onChange">) {
  return (
    <StoryGrid columns={["off", "on"]}>
      {variants.map((variant) => (
        <StoryGridRow key={variant} label={variant}>
          {states.map((isSelected) => (
            <IconToggleButton
              aria-label="Crop"
              isDisabled={isDisabled}
              isSelected={isSelected}
              key={String(isSelected)}
              onChange={onChange}
              variant={variant}
            >
              <Crop />
            </IconToggleButton>
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

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
    variant: {
      control: "inline-radio",
      options: variants,
      table: { defaultValue: { summary: "ghost" } },
    },
  },
  args: {
    "aria-label": "Crop",
    children: <Crop />,
    isDisabled: false,
    isSelected: false,
    onChange: fn(),
    variant: "ghost",
  },
  component: IconToggleButton,
  parameters: {
    controls: {
      include: ["aria-label", "isDisabled", "isSelected", "variant"],
    },
    layout: "centered",
  },
  // A press updates the isSelected control, so the two never disagree.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <IconToggleButton
        {...args}
        onChange={(isSelected) => {
          args.onChange?.(isSelected);
          updateArgs({ isSelected });
        }}
      />
    );
  },
  title: "Primitives/Icon Toggle Button",
} satisfies Meta<typeof IconToggleButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange }) => <VariantGrid onChange={onChange} />,
};

// With `off`, the glyph itself says which state the toggle is in, so being on
// keeps the button's look rather than filling it. A live toggle, using the
// shared render, so a press plays the change from one glyph to the other.
export const WithOffIcon: Story = {
  args: { "aria-label": "Play", children: <Pause />, off: <Play /> },
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange }) => <VariantGrid isDisabled onChange={onChange} />,
};

export const Island: Story = { ...Variants, decorators: [onIsland()] };
