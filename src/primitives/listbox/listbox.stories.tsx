// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ListBoxItem } from "../listbox-item/listbox-item";

import { ListBox } from "./listbox";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A list is drawn in a native panel, so it is shown bare here; in the native
// Storybook it sits on the panel's own material.
const flavours = ["Chocolate", "Strawberry", "Vanilla", "Pistachio"];

const meta = {
  argTypes: {
    selectionMode: {
      control: "inline-radio",
      options: ["single", "multiple"],
      table: { defaultValue: { summary: "single" } },
    },
  },
  args: {
    "aria-label": "Flavour",
    children: flavours.map((flavour) => (
      <ListBoxItem id={flavour} key={flavour} textValue={flavour}>
        {flavour}
      </ListBoxItem>
    )),
    className: "w-48",
    defaultSelectedKeys: ["Strawberry"],
    onSelectionChange: fn(),
    selectionMode: "single",
  },
  component: ListBox,
  parameters: {
    controls: { include: ["selectionMode"] },
    layout: "centered",
  },
  title: "Primitives/Listbox",
} satisfies Meta<typeof ListBox>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithoutSelection: Story = {
  args: {
    "aria-label": "Project",
    children: ["Rename", "Duplicate", "Show in Finder"].map((action) => (
      <ListBoxItem id={action} key={action} showsSelection={false}>
        {action}
      </ListBoxItem>
    )),
    defaultSelectedKeys: [],
    selectionMode: "none",
  },
  parameters: { controls: { disable: true } },
};

export const Disabled: Story = {
  args: { disabledKeys: ["Vanilla", "Pistachio"] },
  parameters: { controls: { disable: true } },
};

export const Empty: Story = {
  args: { children: [] },
  parameters: { controls: { disable: true } },
};

export const LongValue: Story = {
  args: {
    children: (
      <ListBoxItem id="long" textValue="Long value">
        A label far too long for the list it sits in
      </ListBoxItem>
    ),
    defaultSelectedKeys: ["long"],
  },
  parameters: { controls: { disable: true } },
};
