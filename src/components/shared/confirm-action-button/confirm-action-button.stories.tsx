// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Trash2 } from "lucide-react";

import { ConfirmActionButton } from "./confirm-action-button";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    armedClassName:
      "bg-error-surface text-error data-[hovered]:bg-error-surface-hover data-[pressed]:bg-error-surface-pressed",
    armedIcon: <Check />,
    armedLabel: "Confirm discarding",
    idleIcon: <Trash2 />,
    idleLabel: "Discard recording",
    onConfirm: () => undefined,
  },
  component: ConfirmActionButton,
  parameters: { layout: "centered" },
  title: "Components/Confirm Action Button",
} satisfies Meta<typeof ConfirmActionButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Disabled: Story = { args: { isDisabled: true } };

export const Text: Story = { args: { variant: "text" } };

/** The confirming label is the wider one, so the width never changes. */
export const TextDisabled: Story = {
  args: { isDisabled: true, variant: "text" },
};
