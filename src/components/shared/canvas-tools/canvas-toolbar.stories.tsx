// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, RotateCcw, X } from "lucide-react";

import { Button } from "../../base/button/button";
import { IconButton } from "../../base/button/icon-button";

import { CanvasToolbar } from "./canvas-toolbar";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  component: CanvasToolbar,
  parameters: { layout: "centered" },
  title: "Legacy/Canvas Toolbar",
} satisfies Meta<typeof CanvasToolbar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    children: (
      <>
        <Button size="compact" variant="ghost">
          <Copy size={15} />
          Copy
        </Button>
        <IconButton aria-label="Reset" size="compact">
          <RotateCcw size={15} />
        </IconButton>
        <IconButton aria-label="Close" size="compact">
          <X size={15} />
        </IconButton>
      </>
    ),
  },
};
