// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";
import { fn } from "storybook/test";

import { onIsland } from "../../storybook/on-island";
import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { ConfirmActionButton } from "./confirm-action-button";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    variant: {
      control: "inline-radio",
      options: ["solid", "ghost"],
      table: { defaultValue: { summary: "solid" } },
    },
  },
  args: {
    confirmLabel: "Discard",
    isDisabled: false,
    label: "Discard recording",
    onConfirm: fn(),
    variant: "solid",
  },
  component: ConfirmActionButton,
  parameters: {
    controls: { include: ["isDisabled", "variant"] },
    layout: "centered",
  },
  render: (args) => <ConfirmActionButton {...args} icon={<Trash2 />} />,
  title: "Composites/Confirm Action Button",
} satisfies Meta<typeof ConfirmActionButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// The icon button and the text button, solid and ghost, each at rest and
// armed; armed is the primary button.
function StateGrid({
  isDisabled,
  onConfirm,
}: {
  isDisabled?: boolean;
  onConfirm?: () => void;
}) {
  return (
    <StoryGrid columns={["at rest", "armed"]}>
      {[true, false].flatMap((hasIcon) =>
        (["solid", "ghost"] as const).map((variant) => (
          <StoryGridRow
            key={`${String(hasIcon)}-${variant}`}
            label={`${hasIcon ? "icon" : "text"}, ${variant}`}
          >
            {[false, true].map((isArmed) => (
              <ConfirmActionButton
                confirmLabel="Discard"
                icon={hasIcon ? <Trash2 /> : undefined}
                isArmedAtFirst={isArmed}
                isDisabled={isDisabled}
                key={String(isArmed)}
                label="Discard recording"
                onConfirm={onConfirm}
                variant={variant}
              />
            ))}
          </StoryGridRow>
        )),
      )}
    </StoryGrid>
  );
}

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onConfirm }) => <StateGrid onConfirm={onConfirm} />,
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onConfirm }) => <StateGrid isDisabled onConfirm={onConfirm} />,
};

export const Island: Story = { ...States, decorators: [onIsland()] };
