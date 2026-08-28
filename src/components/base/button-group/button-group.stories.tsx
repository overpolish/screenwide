// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";
import { Camera, Keyboard, Mic } from "lucide-react";

import { IconToggleButton } from "../button/icon-button";

import { ButtonGroup } from "./button-group";

const meta = {
  component: ButtonGroup,
  title: "Primitives/Button Group",
} satisfies Meta<typeof ButtonGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    "aria-label": "Recording inputs",
    children: (
      <>
        <IconToggleButton aria-label="Microphone">
          <Mic />
        </IconToggleButton>
        <IconToggleButton aria-label="Camera">
          <Camera />
        </IconToggleButton>
        <IconToggleButton aria-label="Keyboard shortcuts">
          <Keyboard />
        </IconToggleButton>
      </>
    ),
    className: "gap-tight",
  },
};
