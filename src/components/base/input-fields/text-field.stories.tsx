// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TextField } from "./text-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: { defaultValue: "Untitled recording", label: "File name" },
  component: TextField,
  parameters: { layout: "centered" },
  title: "Primitives/Text Field",
} satisfies Meta<typeof TextField>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Default: Story = {};
export const Compact: Story = { args: { size: "compact" } };
export const Disabled: Story = { args: { isDisabled: true } };
