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
    layout: "padded",
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

/** Fine print under the label, in the secondary tone; the box lines up with
 * the label's first line. */
export const WithDescription: Story = {
  args: {
    children: "Bake cursor into recording",
    defaultSelected: true,
    description: "Dynamic Screenwide cursor",
  },
};
