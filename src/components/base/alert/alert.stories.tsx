// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { Alert } from "./alert";

import type { Meta, StoryObj } from "@storybook/react-vite";

const colors: ComponentProps<typeof Alert>["color"][] = ["neutral", "error"];

const meta = {
  argTypes: {
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
    controls: { exclude: ["children", "className", "icon"] },
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

export const Paragraph: Story = {
  args: {
    children:
      "Screenwide could not access this recording source. Check that the source is still available, then try selecting it again.",
  },
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
};

export const Colors: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="gap-section flex w-80 flex-col">
      {colors.map((color) => (
        <Alert key={color} {...args} color={color}>
          {color?.[0].toUpperCase()}
          {color?.slice(1)} alert
        </Alert>
      ))}
    </div>
  ),
};
