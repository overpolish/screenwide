// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../../storybook/story-grid";

import { HotkeyField } from "./hotkey-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const DEFAULT_SHORTCUT = "CommandOrControl+Shift+KeyR";

const meta = {
  argTypes: {
    captureMode: {
      control: "inline-radio",
      options: ["shortcut", "single-control", "local-shortcut"],
      table: { defaultValue: { summary: "shortcut" } },
    },
    isClearable: {
      control: "boolean",
      table: { defaultValue: { summary: "true" } },
    },
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    value: { control: "text" },
  },
  args: {
    "aria-label": "Show recording controls",
    captureMode: "shortcut",
    isClearable: true,
    isDisabled: false,
    onCaptureChange: fn(),
    onChange: fn(),
    value: DEFAULT_SHORTCUT,
  },
  component: HotkeyField,
  parameters: {
    controls: {
      include: ["captureMode", "isClearable", "isDisabled", "value"],
    },
    layout: "centered",
  },
  // A recorded shortcut lands in the args, as Settings would save it.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <HotkeyField
        {...args}
        onChange={(value) => {
          args.onChange(value);
          updateArgs({ value });
        }}
      />
    );
  },
  title: "Features/Settings/Hotkey Field",
} satisfies Meta<typeof HotkeyField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Set, not set, and changed from its default, which offers the reset.
function StateGrid({ isDisabled }: { isDisabled?: boolean }) {
  const rows = [
    { label: "set", value: DEFAULT_SHORTCUT },
    { label: "not set", value: null },
    { label: "changed", value: "CommandOrControl+Alt+KeyS" },
  ];
  return (
    <StoryGrid>
      {rows.map(({ label, value }) => (
        <StoryGridRow key={label} label={label}>
          <HotkeyField
            aria-label="Show recording controls"
            defaultValue={DEFAULT_SHORTCUT}
            isDisabled={isDisabled}
            onChange={() => undefined}
            value={value}
          />
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: () => <StateGrid />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: () => <StateGrid isDisabled />,
};
