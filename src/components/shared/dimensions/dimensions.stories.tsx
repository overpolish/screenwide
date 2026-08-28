// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";

import { Dimensions } from "./dimensions";

const meta = {
  component: Dimensions,
  parameters: {
    layout: "centered",
  },
  title: "Components/Dimensions",
} satisfies Meta<typeof Dimensions>;

export default meta;
type Story = StoryObj<typeof meta>;

/* --------------------------------- Stories -------------------------------- */
export const Default: Story = {};
