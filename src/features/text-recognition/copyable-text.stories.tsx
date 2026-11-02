// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../storybook/feature-story-stage";

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
  decorators: [featureStage({ width: 384 })],
  parameters: {
    controls: { include: ["value"] },
    layout: "fullscreen",
  },
  title: "Legacy/Text Recognition/Copyable Text",
} satisfies Meta<typeof CopyableText>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Empty: Story = {
  args: { value: "" },
  parameters: { controls: { disable: true } },
};
