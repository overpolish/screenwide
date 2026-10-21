// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop, MousePointer2, Type } from "lucide-react";
import { ComponentProps, useState } from "react";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";
import { IconToggleButton } from "../button/icon-button";

import { Toolbar } from "./toolbar";

import type { Meta, StoryObj } from "@storybook/react-vite";

const orientations: NonNullable<
  ComponentProps<typeof Toolbar>["orientation"]
>[] = ["horizontal", "vertical"];

const toolList = [
  { icon: <MousePointer2 />, id: "select", label: "Select" },
  { icon: <Crop />, id: "crop", label: "Crop" },
  { icon: <Type />, id: "text", label: "Text" },
];

// One choice, as the editor's toolbars are: choosing a tool lets go of the
// last, and pressing the chosen one again keeps it.
function Tools() {
  const [chosen, setChosen] = useState("select");
  return toolList.map(({ icon, id, label }) => (
    <IconToggleButton
      aria-label={label}
      isSelected={chosen === id}
      key={id}
      onChange={() => {
        setChosen(id);
      }}
    >
      {icon}
    </IconToggleButton>
  ));
}

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
    children: <Tools />,
    orientation: "horizontal",
  },
  component: Toolbar,
  parameters: {
    controls: { exclude: ["aria-label", "children", "className", "style"] },
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
