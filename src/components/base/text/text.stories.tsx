// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Text } from "./text";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    as: { control: "inline-radio", options: ["p", "span"] },
    variant: { control: "inline-radio", options: ["body", "help"] },
  },
  args: {
    children: "Screenwide needs the following permissions.",
    variant: "body",
  },
  component: Text,
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { include: ["children", "variant", "as"] },
    layout: "centered",
  },
  title: "Primitives/Text",
} satisfies Meta<typeof Text>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Body: Story = {};

export const Help: Story = {
  args: {
    children:
      "Choose where new recordings are saved. You can change this location for each export.",
    variant: "help",
  },
};
