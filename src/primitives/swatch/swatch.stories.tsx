// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Plus } from "lucide-react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { Swatch } from "./swatch";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    isSelected: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    "aria-label": "Orange",
    color: "#ff9f0a",
    isDisabled: false,
    isSelected: false,
    onPress: fn(),
  },
  component: Swatch,
  parameters: {
    controls: { include: ["color", "isDisabled", "isSelected"] },
    layout: "centered",
  },
  // A press toggles the swatch, as a lone one does; in a group the group
  // keeps one chosen.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <Swatch
        {...args}
        onPress={() => {
          args.onPress?.();
          updateArgs({ isSelected: !args.isSelected });
        }}
      />
    );
  },
  title: "Primitives/Swatch",
} satisfies Meta<typeof Swatch>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// The swatch's two looks, a colour and a glyph that stands for an action,
// each at rest and chosen. What a painted swatch shows is Controls' job.
function StateGrid({ isDisabled }: { isDisabled?: boolean }) {
  return (
    <StoryGrid>
      {[false, true].map((isSelected) => (
        <StoryGridRow
          key={String(isSelected)}
          label={isSelected ? "chosen" : "at rest"}
        >
          <span className="flex gap-inline">
            <Swatch
              aria-label="Orange"
              color="#ff9f0a"
              isDisabled={isDisabled}
              isSelected={isSelected}
            />
            <Swatch
              aria-label="Custom"
              isDisabled={isDisabled}
              isSelected={isSelected}
            >
              <Plus />
            </Swatch>
          </span>
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
