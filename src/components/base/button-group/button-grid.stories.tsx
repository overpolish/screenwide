// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { Button } from "../button/button";

import { ButtonGrid } from "./button-group";

const meta = {
  args: {
    "aria-label": "Items",
    children: Array.from({ length: 8 }, (_, index) => (
      <Button key={index}>Item {String(index + 1)}</Button>
    )),
    className: "gap-control w-72",
    columns: 3,
  },
  component: ButtonGrid,
  title: "Primitives/Button Grid",
} satisfies Meta<typeof ButtonGrid>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
