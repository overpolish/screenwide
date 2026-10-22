// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";

import { Badge } from "./badge";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    children: { control: "text" },
    isShown: {
      control: "boolean",
      table: { defaultValue: { summary: "true" } },
    },
  },
  args: {
    children: "Optional",
    isShown: true,
  },
  component: Badge,
  parameters: {
    controls: { exclude: ["className"] },
    layout: "centered",
  },
  title: "Primitives/Badge",
} satisfies Meta<typeof Badge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithIcon: Story = {
  args: {
    children: (
      <>
        <Check />
        Optional
      </>
    ),
  },
};
