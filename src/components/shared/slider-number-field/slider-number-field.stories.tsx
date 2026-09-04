// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { SliderNumberField } from "./slider-number-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    "aria-label": "Amount",
    maxValue: 100,
    minValue: 0,
    onChange: () => undefined,
    value: 25,
  },
  component: SliderNumberField,
  decorators: [
    (Story) => (
      <div className="w-96">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { exclude: ["className", "numberFieldClassName", "onChange"] },
    layout: "centered",
  },
  render: function Render(args) {
    const [draft, setDraft] = useState({
      source: args.value,
      value: args.value,
    });
    // Accept edits from Controls while keeping pointer-rate changes local.
    if (draft.source !== args.value) {
      setDraft({ source: args.value, value: args.value });
    }
    return (
      <SliderNumberField
        {...args}
        onChange={(value) => {
          args.onChange(value);
          setDraft({ source: args.value, value });
        }}
        value={draft.value}
      />
    );
  },
  title: "Components/Slider Number Field",
} satisfies Meta<typeof SliderNumberField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const RestrictedSliderRange: Story = {
  args: {
    "aria-label": "Window gap",
    maxValue: 200,
    rightSection: "px",
    sliderMaxValue: 50,
  },
};

export const AboveSliderRange: Story = {
  args: { ...RestrictedSliderRange.args, value: 120 },
};

export const BelowSliderRange: Story = {
  args: {
    "aria-label": "Offset",
    minValue: -100,
    rightSection: "px",
    sliderMinValue: 0,
    value: -30,
  },
};

export const Units: Story = {
  args: { "aria-label": "Scale", rightSection: "%", value: 75 },
};

export const DecimalStep: Story = {
  args: {
    "aria-label": "Duration",
    maxValue: 10,
    rightSection: "s",
    sliderMaxValue: 5,
    step: 0.1,
    value: 2.5,
  },
};

export const Disabled: Story = {
  args: { ...AboveSliderRange.args, isDisabled: true },
};
