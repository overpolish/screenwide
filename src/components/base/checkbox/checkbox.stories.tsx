// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Checkbox } from "./checkbox";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    children: "Label",
  },
  component: Checkbox,
  parameters: {
    layout: "centered",
  },
  title: "Primitives/Checkbox",
} satisfies Meta<typeof Checkbox>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div className="gap-section flex flex-col items-start">
      <Checkbox>Unchecked</Checkbox>
      <Checkbox isReadOnly isSelected>
        Checked
      </Checkbox>
      <Checkbox isIndeterminate isReadOnly>
        Indeterminate
      </Checkbox>
      <Checkbox isDisabled>Disabled</Checkbox>
      <Checkbox isDisabled isSelected>
        Disabled checked
      </Checkbox>
    </div>
  ),
};

export const WithoutLabel: Story = {
  args: {
    "aria-label": "Selected",
    children: undefined,
    defaultSelected: true,
  },
};
