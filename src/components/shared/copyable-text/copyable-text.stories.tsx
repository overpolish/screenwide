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
    value:
      "WIFI:T:WPA;S:Screenwide Studio;P:correct-horse-battery-staple;;\n\nThis second paragraph demonstrates how longer detected content wraps inside the preview.",
  },
};

export const Empty: Story = {
  args: { value: "" },
};
