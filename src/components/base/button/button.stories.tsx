// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ClipboardCopy, Upload } from "lucide-react";
import { ComponentProps } from "react";

import { StoryGrid, StoryGridRow } from "../../../storybook/story-grid";

import { Button } from "./button";

import type { Meta, StoryObj } from "@storybook/react-vite";

type ButtonProps = ComponentProps<typeof Button>;

// The looks a button can take: a solid one in either colour, or a ghost.
// The primary action is always filled, so there is no primary ghost.
const looks = [
  { label: "neutral", look: { variant: "solid" } },
  { label: "primary", look: { color: "primary", variant: "solid" } },
  { label: "ghost", look: { variant: "ghost" } },
] as const;

function LookGrid({
  children,
  isDisabled,
}: Pick<ButtonProps, "children" | "isDisabled">) {
  return (
    <StoryGrid>
      {looks.map(({ label, look }) => (
        <StoryGridRow key={label} label={label}>
          <Button {...look} isDisabled={isDisabled}>
            {children}
          </Button>
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

const meta = {
  argTypes: {
    children: { control: "text" },
    color: {
      control: "inline-radio",
      options: ["neutral", "primary"],
      table: { defaultValue: { summary: "neutral" } },
    },
    isDisabled: { control: "boolean" },
    variant: {
      control: "inline-radio",
      options: ["solid", "ghost"],
      table: { defaultValue: { summary: "solid" } },
    },
  },
  args: {
    children: "Sign out",
    color: "neutral",
    isDisabled: false,
    variant: "solid",
  },
  component: Button,
  parameters: {
    controls: { exclude: ["ref", "className", "slot"] },
    layout: "centered",
  },
  title: "Primitives/Button",
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: ({ children }) => <LookGrid>{children}</LookGrid>,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ children }) => <LookGrid isDisabled>{children}</LookGrid>,
};

// A window toolbar is the one place a text button carries an icon.
export const WithIcon: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div className="flex items-center gap-2">
      <Button variant="ghost">
        <ClipboardCopy />
        Copy
      </Button>
      <Button color="primary">
        <Upload />
        Export
      </Button>
    </div>
  ),
};
