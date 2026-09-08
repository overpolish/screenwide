// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Text } from "../text/text";

import { TextField } from "./text-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: { defaultValue: "Untitled recording", label: "File name" },
  component: TextField,
  parameters: { layout: "centered" },
  title: "Primitives/Text Field",
} satisfies Meta<typeof TextField>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Disabled: Story = { args: { isDisabled: true } };

export const States: Story = {
  parameters: { controls: { disable: true }, layout: "padded" },
  render: () => (
    <div className="flex flex-col gap-section">
      {(
        [
          { label: "Rest", props: { defaultValue: "Untitled recording" } },
          {
            label: "Empty",
            props: { defaultValue: "", placeholder: "File name" },
          },
          {
            label: "Invalid",
            props: { defaultValue: "Untitled recording", isInvalid: true },
          },
          {
            label: "Disabled",
            props: { defaultValue: "Untitled recording", isDisabled: true },
          },
        ] as const
      ).map((row) => (
        <div className="flex items-center gap-section" key={row.label}>
          <Text className="w-16" variant="footnote">
            {row.label}
          </Text>
          <TextField className="w-48" {...row.props} />
        </div>
      ))}
    </div>
  ),
};
