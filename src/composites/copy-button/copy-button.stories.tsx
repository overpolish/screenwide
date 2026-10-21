// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { CopyButton } from "./copy-button";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    "aria-label": "Copy",
    isDisabled: false,
    onCopy: fn(),
  },
  component: CopyButton,
  parameters: {
    controls: { include: ["isDisabled"] },
    layout: "centered",
  },
  title: "Composites/Copy Button",
} satisfies Meta<typeof CopyButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// A copy that takes a moment, such as a frame rendered first: the glyph
// pulses until it is done.
export const SlowCopy: Story = {
  render: (args) => (
    <CopyButton
      {...args}
      onCopy={() => {
        args.onCopy();
        // The executor form: `Promise.withResolvers` is newer than the
        // TypeScript library this project targets.
        return new Promise((resolve) => {
          setTimeout(resolve, 1500);
        });
      }}
    />
  ),
};

export const Disabled: Story = {
  args: { isDisabled: true },
  parameters: { controls: { disable: true } },
};
