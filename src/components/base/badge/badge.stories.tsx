// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";

import { Badge } from "./badge";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    children: <>Default</>,
  },
  component: Badge,
  parameters: {
    controls: { exclude: ["children", "className"] },
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
        <Check size={12} />
        Default
      </>
    ),
  },
};
