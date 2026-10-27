// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { onIsland } from "../../storybook/on-island";

import { TileChoice } from "./tile-choice";

import type { Meta, StoryObj } from "@storybook/react-vite";

const items = ["Box", "Clean", "Editorial", "Flip", "Glow", "Highlight"].map(
  (name) => ({ id: name.toLowerCase(), label: name }),
);

const meta = {
  argTypes: {
    value: {
      control: "inline-radio",
      options: items.map(({ id }) => id),
    },
  },
  args: {
    "aria-label": "Caption template",
    items,
    onChange: fn(),
    value: "box",
  },
  component: TileChoice,
  decorators: [
    (Story) => (
      <div className="w-72">
        <Story />
      </div>
    ),
  ],
  parameters: { controls: { include: ["value"] }, layout: "centered" },
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <TileChoice
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
  title: "Primitives/Tile Choice",
} satisfies Meta<typeof TileChoice<string>>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Island: Story = { decorators: [onIsland()] };
