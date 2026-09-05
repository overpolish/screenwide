// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PathField } from "./path-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    "aria-label": "Recording location",
    onBrowse: () => undefined,
    value: "/Users/dom/Documents/Screenwide/Recordings",
  },
  component: PathField,
  parameters: { layout: "centered" },
  title: "Components/Path Field",
} satisfies Meta<typeof PathField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const FilePath: Story = {
  args: {
    "aria-label": "Selected file",
    kind: "file",
    value:
      "/Users/dom/Recordings/A very long screen recording filename.screenwide",
  },
};
export const Empty: Story = {
  args: { emptyLabel: "System default", value: null },
};
export const Disabled: Story = { args: { isDisabled: true } };
export const Reset: Story = {
  args: {
    secondaryAction: {
      label: "Use system default",
      onPress: () => undefined,
      type: "reset",
    },
  },
};
