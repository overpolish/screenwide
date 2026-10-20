// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Crop,
  MousePointer2,
  Scissors,
  Type,
  Undo2,
  Upload,
} from "lucide-react";
import { ComponentProps } from "react";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";
import { Button } from "../button/button";
import { IconButton, IconToggleButton } from "../button/icon-button";

import { Toolbar } from "./toolbar";

import type { Meta, StoryObj } from "@storybook/react-vite";

const orientations: NonNullable<
  ComponentProps<typeof Toolbar>["orientation"]
>[] = ["horizontal", "vertical"];

const tools = (
  <>
    <IconToggleButton aria-label="Select" isSelected>
      <MousePointer2 />
    </IconToggleButton>
    <IconToggleButton aria-label="Crop">
      <Crop />
    </IconToggleButton>
    <IconToggleButton aria-label="Text">
      <Type />
    </IconToggleButton>
  </>
);

const meta = {
  argTypes: {
    orientation: {
      control: "inline-radio",
      options: orientations,
      table: { defaultValue: { summary: "horizontal" } },
    },
  },
  args: {
    "aria-label": "Tools",
    children: tools,
    orientation: "horizontal",
  },
  component: Toolbar,
  parameters: {
    controls: { exclude: ["children", "className", "style"] },
    layout: "centered",
  },
  title: "Primitives/Toolbar",
} satisfies Meta<typeof Toolbar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Orientations: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {orientations.map((orientation) => (
        <StoryGridRow key={orientation} label={orientation}>
          <Toolbar {...args} orientation={orientation} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

export const TextAndIcons: Story = {
  args: {
    "aria-label": "Timeline",
    // In a window toolbar every text command takes a leading icon.
    children: (
      <>
        <IconButton aria-label="Undo">
          <Undo2 />
        </IconButton>
        <Button variant="ghost">
          <Scissors />
          Split
        </Button>
        <Button color="primary">
          <Upload />
          Export
        </Button>
      </>
    ),
  },
};
