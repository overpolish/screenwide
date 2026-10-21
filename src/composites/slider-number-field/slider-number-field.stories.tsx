// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { SliderNumberField } from "./slider-number-field";

import type { Decorator, Meta, StoryObj } from "@storybook/react-vite";

// The width a tool panel gives it.
const rowWidth: Decorator = (Story) => (
  <div className="w-48">
    <Story />
  </div>
);

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    unit: { control: "text" },
  },
  args: {
    "aria-label": "Radius",
    isDisabled: false,
    maxValue: 100,
    minValue: 0,
    onChange: fn(),
    unit: "%",
    value: 25,
  },
  component: SliderNumberField,
  parameters: {
    controls: { exclude: ["aria-label", "className", "onChange"] },
    layout: "centered",
  },
  render: function Render(args) {
    const [draft, setDraft] = useState({
      source: args.value,
      value: args.value,
    });
    // Takes edits from Controls while keeping pointer-rate changes local.
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
  title: "Composites/Slider Number Field",
} satisfies Meta<typeof SliderNumberField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { decorators: [rowWidth] };

// The slider's soft range is 0 to 50; typing reaches 0 to 200, and below 0
// for the second field's range, with the knob held at its end.
function StateGrid({ isDisabled }: { isDisabled?: boolean }) {
  const shared = { isDisabled, onChange: () => undefined };
  return (
    <StoryGrid>
      <StoryGridRow label="in range">
        <div className="w-48">
          <SliderNumberField
            {...shared}
            aria-label="Gap"
            maxValue={200}
            minValue={0}
            sliderMaxValue={50}
            unit="px"
            value={24}
          />
        </div>
      </StoryGridRow>
      <StoryGridRow label="above slider">
        <div className="w-48">
          <SliderNumberField
            {...shared}
            aria-label="Gap"
            maxValue={200}
            minValue={0}
            sliderMaxValue={50}
            unit="px"
            value={120}
          />
        </div>
      </StoryGridRow>
      <StoryGridRow label="below slider">
        <div className="w-48">
          <SliderNumberField
            {...shared}
            aria-label="Volume"
            maxValue={12}
            minValue={-60}
            sliderMinValue={-24}
            unit="dB"
            value={-36}
          />
        </div>
      </StoryGridRow>
    </StoryGrid>
  );
}

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: () => <StateGrid />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: () => <StateGrid isDisabled />,
};
