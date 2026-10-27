// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { onIsland } from "../../storybook/on-island";

import { PositionGrid } from "./position-grid";

import type { Meta, StoryObj } from "@storybook/react-vite";

const items = [
  { id: "top-left", label: "Top left" },
  { id: "top", label: "Top" },
  { id: "top-right", label: "Top right" },
  { id: "left", label: "Left" },
  { id: "center", label: "Centre" },
  { id: "right", label: "Right" },
  { id: "bottom-left", label: "Bottom left" },
  { id: "bottom", label: "Bottom" },
  { id: "bottom-right", label: "Bottom right" },
];

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    isDisabled: false,
    items,
    onChange: fn(),
    value: "bottom",
  },
  component: PositionGrid,
  parameters: {
    controls: { include: ["isDisabled"] },
    layout: "centered",
  },
  // A press moves the value, so the fill slides there.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <PositionGrid
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
  title: "Primitives/Position Grid",
} satisfies Meta<typeof PositionGrid<string>>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Disabled: Story = {
  args: { isDisabled: true },
  parameters: { controls: { disable: true } },
};

export const Island: Story = { decorators: [onIsland()] };
