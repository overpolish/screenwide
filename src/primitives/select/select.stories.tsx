// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Film } from "lucide-react";
import { ComponentProps } from "react";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";
import { ListBoxItem } from "../listbox-item/listbox-item";

import { Select } from "./select";

import type { Meta, StoryObj } from "@storybook/react-vite";

type SelectProps = ComponentProps<typeof Select>;

const rates = [
  { id: "24", label: "24 fps" },
  { id: "30", label: "30 fps" },
  { id: "60", label: "60 fps" },
  { id: "long", label: "60 fps, interpolated from a 30 fps recording" },
];

// The meta's Select is generic over any object, so each item is read back as
// a rate.
const options = (item: object) => {
  const rate = item as (typeof rates)[number];
  return (
    <ListBoxItem id={rate.id} textValue={rate.label}>
      {rate.label}
    </ListBoxItem>
  );
};

// A value, the placeholder before one is chosen, a leading glyph, and a
// value too long for the trigger.
const states: { label: string; props: Partial<SelectProps> }[] = [
  { label: "value", props: { selectedKey: "30" } },
  { label: "placeholder", props: { selectedKey: null } },
  { label: "icon", props: { leftSection: <Film />, selectedKey: "30" } },
  { label: "long value", props: { selectedKey: "long" } },
];

function StateGrid({
  isDisabled,
  onPress,
}: Pick<SelectProps, "isDisabled" | "onPress">) {
  return (
    <StoryGrid>
      {states.map(({ label, props }) => (
        <StoryGridRow key={label} label={label}>
          <Select
            {...props}
            aria-label="Frame rate"
            className="w-48"
            isDisabled={isDisabled}
            items={rates}
            onPress={onPress}
            placeholder="Choose a rate"
          >
            {options}
          </Select>
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

// The list opens in the native popup panel, which a story has none of, so a
// press here only reports itself.
const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    label: { control: "text" },
    placeholder: { control: "text" },
  },
  args: {
    children: options,
    className: "w-48",
    defaultSelectedKey: "30",
    isDisabled: false,
    items: rates,
    label: "Frame rate",
    onPress: fn(),
    placeholder: "Choose a rate",
  },
  component: Select,
  parameters: {
    controls: { include: ["isDisabled", "label", "placeholder"] },
    layout: "centered",
  },
  title: "Primitives/Select",
} satisfies Meta<typeof Select>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress }) => <StateGrid onPress={onPress} />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onPress }) => <StateGrid isDisabled onPress={onPress} />,
};
