// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { HotkeyField } from "./hotkey-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    "aria-label": "Show recording bar shortcut",
    onChange: () => undefined,
    value: "CommandOrControl+Shift+KeyR",
  },
  component: HotkeyField,
  parameters: { layout: "centered" },
  render: function Render(args) {
    const [draft, setDraft] = useState({
      source: args.value,
      value: args.value,
    });
    if (draft.source !== args.value)
      setDraft({ source: args.value, value: args.value });
    return (
      <HotkeyField
        {...args}
        onChange={(value) => {
          setDraft({ source: args.value, value });
          args.onChange(value);
        }}
        value={draft.value}
      />
    );
  },
  title: "Components/Hotkey Field",
} satisfies Meta<typeof HotkeyField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Empty: Story = { args: { value: null } };
export const Disabled: Story = { args: { isDisabled: true } };

export const SingleKey: Story = {
  args: {
    "aria-label": "Activation key",
    captureMode: "single-control",
    value: "KeyG",
  },
};
export const SingleModifier: Story = {
  args: {
    "aria-label": "Mouse control",
    captureMode: "single-control",
    isClearable: false,
    value: "AltLeft",
  },
};
export const MouseButton: Story = {
  args: {
    "aria-label": "Mouse control",
    captureMode: "single-control",
    isClearable: false,
    value: "MouseMiddle",
  },
};
