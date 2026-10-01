// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTrackLanesPreview } from "./recording-track-lanes-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

function TimelinePreview() {
  return (
    <div className="w-[760px]">
      <RecordingTrackLanesPreview />
    </div>
  );
}

const meta = {
  component: TimelinePreview,
  parameters: { layout: "centered" },
  title: "Features/Editor/Timeline",
} satisfies Meta<typeof TimelinePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ZoomAndPan: Story = {};
