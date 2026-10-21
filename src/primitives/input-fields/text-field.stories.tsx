// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FileText } from "lucide-react";
import { ComponentProps } from "react";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { TextField } from "./text-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

type TextFieldProps = ComponentProps<typeof TextField>;

// At rest with a value, empty with its placeholder, and invalid.
const states: { label: string; props: Partial<TextFieldProps> }[] = [
  { label: "rest", props: { defaultValue: "Untitled recording" } },
  { label: "empty", props: { defaultValue: "", placeholder: "File name" } },
  {
    label: "invalid",
    props: { defaultValue: "Untitled recording", isInvalid: true },
  },
];

function StateGrid({
  isDisabled,
  onChange,
}: Pick<TextFieldProps, "isDisabled" | "onChange">) {
  return (
    <StoryGrid>
      {states.map(({ label, props }) => (
        <StoryGridRow key={label} label={label}>
          <TextField
            {...props}
            aria-label="File name"
            className="w-56"
            isDisabled={isDisabled}
            onChange={onChange}
          />
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

const meta = {
  argTypes: {
    defaultValue: { control: "text" },
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    isInvalid: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    label: { control: "text" },
    placeholder: { control: "text" },
  },
  args: {
    className: "w-56",
    defaultValue: "Untitled recording",
    isDisabled: false,
    isInvalid: false,
    label: "File name",
    onChange: fn(),
    placeholder: "File name",
  },
  component: TextField,
  parameters: {
    controls: {
      include: [
        "defaultValue",
        "isDisabled",
        "isInvalid",
        "label",
        "placeholder",
      ],
    },
    layout: "centered",
  },
  title: "Primitives/Text Field",
} satisfies Meta<typeof TextField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithIcon: Story = {
  args: { icon: <FileText /> },
};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange }) => <StateGrid onChange={onChange} />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange }) => <StateGrid isDisabled onChange={onChange} />,
};
