// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowUp } from "lucide-react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { IslandDial } from "./island-dial";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    value: { control: { max: 359, min: 0, step: 1, type: "range" } },
  },
  args: {
    "aria-label": "Aim",
    children: <ArrowUp className="size-icon" />,
    onChange: fn(),
    value: 45,
  },
  component: IslandDial,
  decorators: [
    // On the island's black, which is always drawn dark.
    (Story) => (
      <div className="dark rounded-full bg-island p-1 text-fg">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { include: ["value"] },
    layout: "centered",
  },
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <IslandDial
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
  title: "Primitives/Island Dial",
} satisfies Meta<typeof IslandDial>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
