// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { StoryGrid, StoryGridRow } from "../../../storybook/story-grid";
import { Button } from "../button/button";

import { Alert } from "./alert";

import type { Meta, StoryObj } from "@storybook/react-vite";

const colors: NonNullable<ComponentProps<typeof Alert>["color"]>[] = [
  "neutral",
  "error",
];

const meta = {
  argTypes: {
    children: { control: "text" },
    color: {
      control: "inline-radio",
      options: colors,
      table: { defaultValue: { summary: "neutral" } },
    },
  },
  args: {
    children: "No additional options are available for this recording.",
    color: "neutral",
  },
  component: Alert,
  parameters: {
    controls: { exclude: ["action", "className", "icon"] },
    layout: "centered",
  },
  title: "Primitives/Alert",
} satisfies Meta<typeof Alert>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
};

export const WithAction: Story = {
  args: {
    action: <Button variant="ghost">Undo</Button>,
    children: "Moved 3 projects to the Trash.",
  },
  decorators: [
    (Story) => (
      <div className="w-96">
        <Story />
      </div>
    ),
  ],
};

export const Colors: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {colors.map((color) => (
        <StoryGridRow key={color} label={color}>
          <div className="w-80">
            <Alert {...args} color={color} />
          </div>
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
