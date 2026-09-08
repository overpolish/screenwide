// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, X } from "lucide-react";

import { ConfirmActionButton } from "../confirm-action-button/confirm-action-button";

import { ProgressPanel } from "./progress-panel";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** Cancelling running work is a two-press confirmation, as it is in the app. */
const cancel = (
  <ConfirmActionButton
    armedIcon={<Check />}
    armedLabel="Confirm cancel"
    idleIcon={<X />}
    idleLabel="Cancel"
    variant="text"
  />
);

const meta = {
  args: { label: "Saving recording", progress: 42 },
  component: ProgressPanel,
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
  parameters: { layout: "padded" },
  title: "Components/Progress Panel",
} satisfies Meta<typeof ProgressPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Determinate: Story = {
  args: {
    action: cancel,
    secondary: "About 2 min remaining",
  },
};

/** Work that cannot report how far it has come carries the wait in the bar. */
export const Indeterminate: Story = {
  args: {
    action: cancel,
    label: "Finalizing recording",
    progress: null,
  },
};
