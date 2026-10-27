// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowUp } from "lucide-react";
import { ComponentProps, useState } from "react";
import { fn } from "storybook/test";

import { onIsland } from "../../storybook/on-island";
import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { AngleDial } from "./angle-dial";

import type { Meta, StoryObj } from "@storybook/react-vite";

const colors: NonNullable<ComponentProps<typeof AngleDial>["color"]>[] = [
  "neutral",
  "red",
  "green",
  "blue",
];

const meta = {
  argTypes: {
    color: {
      control: "inline-radio",
      options: colors,
      table: { defaultValue: { summary: "neutral" } },
    },
  },
  args: {
    "aria-label": "Angle",
    color: "neutral",
    onChange: fn(),
    value: 0,
  },
  component: AngleDial,
  parameters: {
    controls: { exclude: ["aria-label", "className", "onChange"] },
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
      <div className="flex items-center gap-inline">
        <AngleDial
          {...args}
          onChange={(value) => {
            args.onChange(value);
            setDraft({ source: args.value, value });
          }}
          value={draft.value}
        />
        <span className="w-10 text-body text-fg-secondary tabular-nums">
          {draft.value}°
        </span>
      </div>
    );
  },
  title: "Primitives/Angle Dial",
} satisfies Meta<typeof AngleDial>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Disabled: Story = {
  args: { isDisabled: true, value: 45 },
};

export const Colors: Story = {
  args: { value: 45 },
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {colors.map((color) => (
        <StoryGridRow key={color} label={color}>
          <AngleDial {...args} color={color} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

// On the island the dial turns what it is given, here an arrow, an eighth of
// a turn at a time.
export const Island: Story = {
  decorators: [onIsland()],
  render: function Render(args) {
    const [value, setValue] = useState(args.value);
    return (
      <AngleDial
        {...args}
        onChange={(next) => {
          args.onChange(next);
          setValue(next);
        }}
        value={value}
      >
        <ArrowUp aria-hidden className="size-icon" />
      </AngleDial>
    );
  },
};
