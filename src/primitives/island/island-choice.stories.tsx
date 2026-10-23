// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlignCenter, AlignLeft, AlignRight } from "lucide-react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { IslandChoice } from "./island-choice";

import type { Meta, StoryObj } from "@storybook/react-vite";

const items = [
  { content: <AlignLeft className="size-icon" />, id: "start", label: "Start" },
  {
    content: <AlignCenter className="size-icon" />,
    id: "center",
    label: "Centre",
  },
  { content: <AlignRight className="size-icon" />, id: "end", label: "End" },
];

const meta = {
  args: {
    items,
    label: "Alignment",
    onChange: fn(),
    value: "start",
  },
  component: IslandChoice,
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
      <IslandChoice
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
  title: "Primitives/Island Choice",
} satisfies Meta<typeof IslandChoice>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
