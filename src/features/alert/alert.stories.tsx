// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { Alert } from "./alert";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    message:
      "The camera no longer offers 1920 × 1080 at 30 fps. Choose another camera mode in the recording bar, or turn the camera off.",
    onDismiss: () => undefined,
    title: "Recording could not start",
  },
  component: Alert,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={420}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Alert",
} satisfies Meta<typeof Alert>;

export default meta;
type Story = StoryObj<typeof meta>;

export const CameraModeUnavailable: Story = {};

export const ReplayBufferStopped: Story = {
  args: {
    message: "None of the selected applications are currently available",
    title: "Replay buffer could not turn on",
  },
};
