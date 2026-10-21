// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { SearchField } from "./search-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

type SearchFieldProps = ComponentProps<typeof SearchField>;

// Empty with its placeholder, and with a query, which brings the clear button.
const states: { label: string; props: Partial<SearchFieldProps> }[] = [
  { label: "empty", props: { defaultValue: "" } },
  { label: "query", props: { defaultValue: "demo" } },
];

function StateGrid({
  isDisabled,
  onChange,
  onClear,
}: Pick<SearchFieldProps, "isDisabled" | "onChange" | "onClear">) {
  return (
    <StoryGrid>
      {states.map(({ label, props }) => (
        <StoryGridRow key={label} label={label}>
          <SearchField
            {...props}
            aria-label="Search projects"
            className="w-56"
            isDisabled={isDisabled}
            onChange={onChange}
            onClear={onClear}
            placeholder="Search"
          />
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    placeholder: { control: "text" },
  },
  args: {
    "aria-label": "Search projects",
    className: "w-56",
    isDisabled: false,
    onChange: fn(),
    onClear: fn(),
    onSubmit: fn(),
    placeholder: "Search",
  },
  component: SearchField,
  parameters: {
    controls: { include: ["isDisabled", "placeholder"] },
    layout: "centered",
  },
  title: "Primitives/Search Field",
} satisfies Meta<typeof SearchField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange, onClear }) => (
    <StateGrid onChange={onChange} onClear={onClear} />
  ),
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange, onClear }) => (
    <StateGrid isDisabled onChange={onChange} onClear={onClear} />
  ),
};
