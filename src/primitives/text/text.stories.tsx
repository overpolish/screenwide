// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { Text } from "./text";

import type { Meta, StoryObj } from "@storybook/react-vite";

const variants = ["title", "heading", "body", "label", "caption"] as const;

const meta = {
  argTypes: {
    as: {
      control: "inline-radio",
      options: ["p", "span", "h1", "h2", "h3", "label"],
      table: { defaultValue: { summary: "p" } },
    },
    variant: {
      control: "inline-radio",
      options: variants,
      table: { defaultValue: { summary: "body" } },
    },
  },
  args: {
    as: "p",
    children: "Recordings are kept on this computer.",
    variant: "body",
  },
  component: Text,
  parameters: {
    controls: { include: ["as", "children", "variant"] },
    layout: "centered",
  },
  title: "Primitives/Text",
} satisfies Meta<typeof Text>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {variants.map((variant) => (
        <StoryGridRow key={variant} label={variant}>
          <Text {...args} variant={variant} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
