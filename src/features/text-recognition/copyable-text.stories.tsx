// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { CopyableText } from "./copyable-text";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: { value: { control: "text" } },
  args: {
    copyLabel: "Copy detected content",
    label: "Detected content",
    onCopy: fn(),
    value: "https://screenwide.app",
  },
  component: CopyableText,
  decorators: [
    (Story) => (
      <div className="w-96">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { include: ["value"] },
    layout: "centered",
  },
  title: "Legacy/Text Recognition/Copyable Text",
} satisfies Meta<typeof CopyableText>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const LongContent: Story = {
  args: {
    value: Array.from(
      { length: 12 },
      (_, index) =>
        `Line ${String(index + 1)}: WIFI:T:WPA;S:Screenwide Studio;P:correct-horse-battery-staple;; and a longer trailing sentence so that wrapping is exercised as well as scrolling.`,
    ).join("\n\n"),
  },
  parameters: { controls: { disable: true } },
};

export const Empty: Story = {
  args: { value: "" },
  parameters: { controls: { disable: true } },
};
