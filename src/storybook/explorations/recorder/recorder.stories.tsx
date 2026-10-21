// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../feature-story-stage";

import { PointingOverlay } from "./pointing-overlay";
import { RecorderOptions } from "./recorder-options";
import { RecorderPanel } from "./recorder-panel";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  component: RecorderPanel,
  parameters: { layout: "centered" },
  title: "Explorations/Recorder",
} satisfies Meta<typeof RecorderPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const InputsAsTiles: Story = { args: { inputLayout: "tiles" } };

export const TwoDisplays: Story = { args: { displayCount: 2 } };

export const PointingAtWindow: Story = {
  decorators: [featureStage({ height: 625, width: 1000 })],
  parameters: { layout: "fullscreen" },
  render: () => <PointingOverlay target="window" />,
};

export const WindowChosen: Story = { args: { initialTarget: "window" } };

export const DraggingArea: Story = {
  decorators: [featureStage({ height: 625, width: 1000 })],
  parameters: { layout: "fullscreen" },
  render: () => <PointingOverlay target="area" />,
};

export const AreaChosen: Story = { args: { initialTarget: "area" } };

export const CameraOnly: Story = {
  args: { initialInputs: { camera: true }, initialTarget: "none" },
};

export const SoundOnly: Story = { args: { initialTarget: "none" } };

export const NoMicrophone: Story = {
  args: { availability: { microphone: "no-device" } },
};

export const PermissionsMissing: Story = {
  args: {
    availability: { camera: "not-allowed", microphone: "not-allowed" },
    canRecordScreen: false,
  },
};

export const ReplayOn: Story = { args: { isReplayOn: true } };

// The menu opens from Options, at the panel's foot.
export const Options: Story = {
  args: { isOptionsOpen: true },
  decorators: [
    (Story) => (
      <div className="flex flex-col items-end">
        <Story />
        <div className="-mt-inset me-inset">
          <RecorderOptions />
        </div>
      </div>
    ),
  ],
};
