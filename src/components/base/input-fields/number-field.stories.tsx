// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { Ruler } from "lucide-react";

import { NumberField } from "./number-field";

const sizes: React.ComponentProps<typeof NumberField>["size"][] = [
  "default",
  "compact",
] as const;

const meta = {
  argTypes: {
    showSteppers: { control: "boolean" },
    size: {
      control: "inline-radio",
      options: sizes,
      table: { defaultValue: { summary: "default" } },
    },
  },
  args: {
    defaultValue: 5,
    label: "Amount",
    maxValue: 100,
    minValue: 0,
    step: 1,
  },
  component: NumberField,
  parameters: {
    controls: {
      exclude: ["className", "leftSection", "rightSection"],
    },
    layout: "centered",
  },
  title: "Primitives/Number Field",
} satisfies Meta<typeof NumberField>;

export default meta;

type Story = StoryObj<typeof meta>;

/* --------------------------------- Stories -------------------------------- */
export const Default: Story = {
  args: {
    showSteppers: true,
    size: "default",
  },
};

export const Sizes: Story = {
  parameters: {
    controls: { disable: true },
  },
  render: (args) => (
    <div className="flex gap-2 items-center">
      {sizes.map((size) => (
        <NumberField key={size} size={size} {...args} />
      ))}
    </div>
  ),
};

export const WithoutSteppers: Story = {
  args: {
    label: undefined,
    showSteppers: false,
  },
  parameters: { controls: { disable: true } },
};

export const Sections: Story = {
  args: {
    leftSection: <Ruler size={18} />,
    rightSection: "px",
    showSteppers: false,
  },
  parameters: { controls: { include: ["showSteppers", "size"] } },
};

export const Disabled: Story = {
  args: { isDisabled: true },
};
