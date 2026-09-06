// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, X } from "lucide-react";

import { Keyboard, Shortcut } from "../../base/keyboard/keyboard";
import { ConfirmActionButton } from "../confirm-action-button/confirm-action-button";

import { ProgressPanel } from "./progress-panel";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** Cancelling running work is a two-press confirmation, as it is in the app. */
const cancel = (
  <ConfirmActionButton
    armedClassName="bg-error-surface text-error data-[hovered]:bg-error-surface-hover data-[pressed]:bg-error-surface-pressed"
    armedIcon={<Check />}
    armedLabel="Confirm cancel"
    idleIcon={<X />}
    idleLabel="Cancel"
    size="compact"
    variant="text"
  />
);

const meta = {
  args: { label: "Saving recording", progress: 42 },
  component: ProgressPanel,
  parameters: { layout: "centered" },
  title: "Components/Progress Panel",
} satisfies Meta<typeof ProgressPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Determinate: Story = {
  args: {
    action: cancel,
    secondary: "About 2 min remaining",
    size: "large",
  },
};

/** Work that cannot report how far it has come carries the wait in the ring. */
export const Indeterminate: Story = {
  args: {
    action: cancel,
    label: "Finalizing recording",
    progress: null,
    size: "large",
  },
};

export const Row: Story = {
  args: {
    label: "Capturing",
    orientation: "row",
    progress: null,
    progressLabel: "Scrolling capture progress",
    secondary: (
      <span className="gap-control flex items-center whitespace-nowrap">
        <Shortcut>
          <Keyboard>Esc</Keyboard>
        </Shortcut>
        <span>to cancel</span>
      </span>
    ),
  },
};
