// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CopyableText } from "./copyable-text";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    label: "Detected content",
    onCopy: () => undefined,
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
    controls: { exclude: ["className", "onCopy"] },
    layout: "centered",
  },
  title: "Components/Copyable Text",
} satisfies Meta<typeof CopyableText>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Short: Story = {};

export const Multiline: Story = {
  args: {
    value: Array.from(
      { length: 12 },
      (_, index) =>
        `Line ${String(index + 1)}: WIFI:T:WPA;S:Screenwide Studio;P:correct-horse-battery-staple;; and a longer trailing sentence so that wrapping is exercised as well as scrolling.`,
    ).join("\n\n"),
  },
};

export const Empty: Story = {
  args: { value: "" },
};
