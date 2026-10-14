// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ColorChip } from "./color-chip";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: { label: "Accent", onPress: fn(), value: "#ffd60a" },
  component: ColorChip,
  parameters: { layout: "centered" },
  title: "Primitives/Colour Chip",
} satisfies Meta<typeof ColorChip>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Disabled: Story = { args: { isDisabled: true } };
