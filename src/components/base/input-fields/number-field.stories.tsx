// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { Ruler } from "lucide-react";

import { Text } from "../text/text";

import { NumberField } from "./number-field";

const meta = {
  argTypes: {
    showSteppers: { control: "boolean" },
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
  },
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
    leftSection: <Ruler />,
    rightSection: "px",
    showSteppers: false,
  },
  parameters: { controls: { include: ["showSteppers"] } },
};

export const Disabled: Story = {
  args: { isDisabled: true },
};

export const States: Story = {
  parameters: { controls: { disable: true }, layout: "padded" },
  render: () => (
    <div className="flex flex-col gap-section">
      {(
        [
          { label: "Rest", props: { defaultValue: 5 } },
          {
            label: "Empty",
            props: { defaultValue: Number.NaN, placeholder: "Amount" },
          },
          { label: "Invalid", props: { defaultValue: 5, isInvalid: true } },
          { label: "Disabled", props: { defaultValue: 5, isDisabled: true } },
        ] as const
      ).map((row) => (
        <div className="flex items-center gap-section" key={row.label}>
          <Text className="w-16" variant="footnote">
            {row.label}
          </Text>
          <NumberField
            className="w-48"
            label="Amount"
            maxValue={100}
            minValue={0}
            step={1}
            {...row.props}
          />
        </div>
      ))}
    </div>
  ),
};
