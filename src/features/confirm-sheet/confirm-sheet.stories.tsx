// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { ConfirmSheet } from "./confirm-sheet";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    cancelLabel: "Cancel",
    confirmLabel: "Delete",
    message:
      "Closing the editor deletes the unsaved recording. This cannot be undone.",
    onCancel: () => undefined,
    onConfirm: () => undefined,
    title: "Delete this recording?",
  },
  component: ConfirmSheet,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={420}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Confirm Sheet",
} satisfies Meta<typeof ConfirmSheet>;

export default meta;
type Story = StoryObj<typeof meta>;

export const DeleteRecording: Story = {};

export const DeleteScreenshot: Story = {
  args: {
    message:
      "Closing the editor deletes the unsaved screenshot. This cannot be undone.",
    title: "Delete this screenshot?",
  },
};
