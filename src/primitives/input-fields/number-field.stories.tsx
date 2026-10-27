// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ruler } from "lucide-react";
import { ComponentProps } from "react";
import { fn } from "storybook/test";

import { onIsland } from "../../storybook/on-island";
import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { NumberField } from "./number-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

type NumberFieldProps = ComponentProps<typeof NumberField>;

// At rest with a value, empty with its placeholder, and invalid.
const states: { label: string; props: Partial<NumberFieldProps> }[] = [
  { label: "rest", props: { defaultValue: 5 } },
  {
    label: "empty",
    props: { defaultValue: Number.NaN, placeholder: "Amount" },
  },
  { label: "invalid", props: { defaultValue: 5, isInvalid: true } },
];

function StateGrid({
  isDisabled,
  onChange,
}: Pick<NumberFieldProps, "isDisabled" | "onChange">) {
  return (
    <StoryGrid>
      {states.map(({ label, props }) => (
        <StoryGridRow key={label} label={label}>
          <NumberField
            {...props}
            aria-label="Amount"
            className="w-32"
            isDisabled={isDisabled}
            maxValue={100}
            minValue={0}
            onChange={onChange}
          />
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
    isInvalid: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    label: { control: "text" },
    maxValue: { control: "number" },
    minValue: { control: "number" },
    step: { control: "number" },
  },
  args: {
    className: "w-32",
    defaultValue: 5,
    isDisabled: false,
    isInvalid: false,
    label: "Amount",
    maxValue: 100,
    minValue: 0,
    onChange: fn(),
    step: 1,
  },
  component: NumberField,
  parameters: {
    controls: {
      include: [
        "isDisabled",
        "isInvalid",
        "label",
        "maxValue",
        "minValue",
        "step",
      ],
    },
    layout: "centered",
  },
  title: "Primitives/Number Field",
} satisfies Meta<typeof NumberField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Sections: Story = {
  args: { leftSection: <Ruler />, rightSection: "px" },
};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange }) => <StateGrid onChange={onChange} />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange }) => <StateGrid isDisabled onChange={onChange} />,
};

export const Island: Story = { ...States, decorators: [onIsland()] };
