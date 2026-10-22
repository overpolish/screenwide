// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic, MicOff } from "lucide-react";

import { IconSwap } from "./icon-swap";

import type { Meta, StoryObj } from "@storybook/react-vite";

const glyphs = ["on", "off"];

const meta = {
  argTypes: {
    swapKey: {
      control: "inline-radio",
      options: glyphs,
      table: { defaultValue: { summary: "on" } },
    },
  },
  args: { children: null, swapKey: "on" },
  component: IconSwap,
  parameters: {
    controls: { include: ["swapKey"] },
    layout: "centered",
  },
  // The glyph follows the key, so picking the other value plays the swap.
  render: ({ swapKey }) => (
    <span className="relative flex size-control-height text-fg">
      <IconSwap swapKey={swapKey}>
        {swapKey === "on" ? (
          <Mic className="size-icon" />
        ) : (
          <MicOff className="size-icon" />
        )}
      </IconSwap>
    </span>
  ),
  title: "Primitives/Icon Swap",
} satisfies Meta<typeof IconSwap>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
