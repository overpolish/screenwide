// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../../../storybook/feature-story-stage";
import { recordedTime, timelineScale } from "../timeline-scale";

import { RangeBand } from "./range-band";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A strip 64 seconds long, 640 px wide, so a second is 10 px.
const WIDTH = 640;

const meta = {
  args: {
    scale: timelineScale(recordedTime(64)),
    span: { end: 30.5, start: 17 },
  },
  component: RangeBand,
  decorators: [featureStage({ width: WIDTH })],
  parameters: { controls: { include: ["span"] } },
  // On a row of the strip's height.
  render: (args) => (
    <div className="relative mt-2 h-11" dir="ltr" style={{ width: WIDTH }}>
      <RangeBand {...args} />
    </div>
  ),
  title: "Features/Editor/Timeline/Range Band",
} satisfies Meta<typeof RangeBand>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
