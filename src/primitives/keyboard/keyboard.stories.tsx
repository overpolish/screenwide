// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { Keyboard, Shortcut } from "./keyboard";

import type { Meta, StoryObj } from "@storybook/react-vite";

const variants = ["keycap", "plain"] as const;

// A modifier, a named key, a letter, a digit and a bracket: on the Mac the
// first two draw as symbols.
const sampleKeys = ["Command", "Escape", "B", "4", "["];
const separator = "+" as const;

const meta = {
  argTypes: {
    children: { control: "text" },
    variant: {
      control: "inline-radio",
      options: variants,
      table: { defaultValue: { summary: "keycap" } },
    },
  },
  args: { children: "Esc", variant: "keycap" },
  component: Keyboard,
  parameters: {
    controls: { include: ["children", "variant"] },
    layout: "centered",
  },
  title: "Primitives/Keyboard",
} satisfies Meta<typeof Keyboard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <StoryGrid columns={sampleKeys.length}>
      {variants.map((variant) => (
        <StoryGridRow key={variant} label={variant}>
          {sampleKeys.map((key) => (
            <Keyboard key={key} variant={variant}>
              {key}
            </Keyboard>
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

export const ShortcutKeys: Story = {
  name: "Shortcut",
  parameters: { controls: { disable: true } },
  render: () => (
    <StoryGrid>
      <StoryGridRow label="separated">
        <Shortcut>
          <Keyboard>Command</Keyboard>
          {separator}
          <Keyboard>Shift</Keyboard>
          {separator}
          <Keyboard>4</Keyboard>
        </Shortcut>
      </StoryGridRow>
      <StoryGridRow label="together">
        <Shortcut>
          <Keyboard>Command</Keyboard>
          <Keyboard>Shift</Keyboard>
          <Keyboard>4</Keyboard>
        </Shortcut>
      </StoryGridRow>
    </StoryGrid>
  ),
};
