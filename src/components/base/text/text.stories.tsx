// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Text } from "./text";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    as: { control: "inline-radio", options: ["p", "span", "h1", "h2", "h3"] },
    variant: {
      control: "inline-radio",
      options: [
        "title",
        "headline",
        "body",
        "subheadline",
        "label",
        "footnote",
      ],
    },
  },
  args: {
    as: "p",
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

export const Subheadline: Story = {
  args: {
    children:
      "Choose where new recordings are saved. You can change this location for each export.",
    variant: "subheadline",
  },
};

export const Scale: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div className="flex flex-col gap-section">
      <Text as="h1" variant="title">
        Title
      </Text>
      <Text as="h2" variant="headline">
        Headline
      </Text>
      <Text variant="body">Body</Text>
      <Text variant="subheadline">Subheadline</Text>
      <Text variant="label">Label</Text>
      <Text variant="footnote">Footnote</Text>
    </div>
  ),
};
