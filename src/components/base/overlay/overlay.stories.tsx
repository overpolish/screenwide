// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock } from "lucide-react";

import { IconButton } from "../button/icon-button";
import { CircularProgress } from "../circular-progress/circular-progress";
import { Text } from "../text/text";

import { Overlay } from "./overlay";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    blur: { control: "inline-radio", options: ["xs", "sm", "md", "lg"] },
  },
  args: { blur: "sm", isOpen: true },
  component: Overlay,
  decorators: [
    (Story) => (
      <div className="p-window-inset relative isolate flex h-40 w-80 transform-gpu items-center justify-center overflow-hidden rounded-window">
        <Text>Recording preview</Text>
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { exclude: ["children", "className"] },
    layout: "centered",
  },
  title: "Primitives/Overlay",
} satisfies Meta<typeof Overlay>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Busy: Story = {
  args: {
    children: (
      <div className="gap-control-inset flex items-center" role="status">
        <CircularProgress
          aria-label="Starting recording"
          isIndeterminate
          size="compact"
        />
        <Text>Starting</Text>
      </div>
    ),
  },
};
export const Locked: Story = {
  args: {
    children: (
      <IconButton aria-label="Open permissions">
        <Lock />
      </IconButton>
    ),
  },
};
export const Contained: Story = { args: { contained: true } };
