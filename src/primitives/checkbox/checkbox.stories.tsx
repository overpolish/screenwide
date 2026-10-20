// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { Checkbox } from "./checkbox";

import type { Meta, StoryObj } from "@storybook/react-vite";

type CheckboxProps = ComponentProps<typeof Checkbox>;

// Unchecked, checked and indeterminate side by side, all with the given args.
function StateGrid(args: CheckboxProps) {
  return (
    <StoryGrid>
      <StoryGridRow label="unchecked">
        <Checkbox {...args} isIndeterminate={false} isSelected={false} />
      </StoryGridRow>
      <StoryGridRow label="checked">
        <Checkbox {...args} isIndeterminate={false} isSelected />
      </StoryGridRow>
      <StoryGridRow label="indeterminate">
        <Checkbox {...args} isIndeterminate />
      </StoryGridRow>
    </StoryGrid>
  );
}

const meta = {
  argTypes: {
    children: { control: "text" },
    description: { control: "text" },
    isDisabled: { control: "boolean" },
    isIndeterminate: { control: "boolean" },
    isSelected: { control: "boolean" },
  },
  args: {
    children: "Open folder after export",
    isDisabled: false,
    isIndeterminate: false,
    isSelected: false,
    onChange: fn(),
  },
  component: Checkbox,
  parameters: { controls: { exclude: ["onChange"] }, layout: "centered" },
  // A click updates the isSelected control, so the two never disagree.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <Checkbox
        {...args}
        onChange={(isSelected) => {
          args.onChange?.(isSelected);
          updateArgs({ isIndeterminate: false, isSelected });
        }}
      />
    );
  },
  title: "Primitives/Checkbox",
} satisfies Meta<typeof Checkbox>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithDescription: Story = {
  args: {
    children: "Delete project after export",
    description: "The recording and its edits are moved to the Trash.",
  },
};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => <StateGrid {...args} isDisabled={false} />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => <StateGrid {...args} isDisabled />,
};
