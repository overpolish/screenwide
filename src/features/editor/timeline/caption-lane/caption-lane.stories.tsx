// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { recordedTime, timelineScale } from "../timeline-scale";

import { CaptionLane } from "./caption-lane";
import { storyCaption } from "./caption-story-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A strip 30 seconds long, 640 px wide, so a second is about 21 px.
const WIDTH = 640;
const scale = timelineScale(recordedTime(30));

const meta = {
  args: {
    isHidden: false,
    lines: [
      storyCaption(1, 9, "Open the app"),
      storyCaption(10, 14.5, "Pick a template, then name it so we find it"),
      storyCaption(16, 18, "Done"),
      storyCaption(19, 29, "Now share it"),
    ],
    onPress: fn(),
    scale,
    width: WIDTH,
  },
  component: CaptionLane,
  decorators: [featureStage({ width: WIDTH })],
  parameters: { controls: { include: ["isHidden", "lines"] } },
  render: (args) => (
    <div className="py-2" style={{ width: WIDTH }}>
      <CaptionLane {...args} />
    </div>
  ),
  title: "Features/Editor/Timeline/Caption Lane",
} satisfies Meta<typeof CaptionLane>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
