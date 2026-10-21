// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop } from "lucide-react";

import { IconButton } from "../../primitives/button/icon-button";

import { NativeTooltipTrigger } from "./native-tooltip-trigger";

import type { Meta, StoryObj } from "@storybook/react-vite";

// The trigger opens the tooltip window, which only exists in the app: in
// `pnpm dev:storybook-native` hovering the button shows it, and in a browser
// nothing does. The window's content is the Tooltip primitive.
const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    tooltip: { control: "text" },
  },
  args: {
    children: null,
    isDisabled: false,
    tooltip: "Crop the recording",
  },
  component: NativeTooltipTrigger,
  parameters: {
    controls: { include: ["isDisabled", "tooltip"] },
    layout: "centered",
  },
  render: (args) => (
    <NativeTooltipTrigger {...args}>
      <IconButton aria-label="Crop" variant="ghost">
        <Crop />
      </IconButton>
    </NativeTooltipTrigger>
  ),
  title: "Composites/Native Tooltip",
} satisfies Meta<typeof NativeTooltipTrigger>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
