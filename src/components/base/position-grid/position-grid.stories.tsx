// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { PositionGrid } from "./position-grid";

import type { Meta, StoryObj } from "@storybook/react-vite";

const items = [
  { id: "top-left", label: "Top left" },
  { id: "top", label: "Top" },
  { id: "top-right", label: "Top right" },
  { id: "left", label: "Left" },
  { id: "center", label: "Centre" },
  { id: "right", label: "Right" },
  { id: "bottom-left", label: "Bottom left" },
  { id: "bottom", label: "Bottom" },
  { id: "bottom-right", label: "Bottom right" },
];

function PositionGridExample({ isDisabled }: { isDisabled?: boolean }) {
  const [value, setValue] = useState("bottom");
  return (
    <PositionGrid
      isDisabled={isDisabled}
      items={items}
      onChange={setValue}
      value={value}
    />
  );
}

const meta = {
  component: PositionGridExample,
  parameters: { layout: "centered" },
  title: "Primitives/Position Grid",
} satisfies Meta<typeof PositionGridExample>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Disabled: Story = { args: { isDisabled: true } };
