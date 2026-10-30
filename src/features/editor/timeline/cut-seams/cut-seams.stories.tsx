// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { timelineScale } from "../timeline-scale";
import { squeezedTime } from "../timeline-story-fixtures";

import { CutSeams, type TimelineCut } from "./cut-seams";

import type { Meta, StoryObj } from "@storybook/react-vite";

const WIDTH = 640;
const DURATION = 60;

type StoryProps = {
  cuts: TimelineCut[];
  onRestore: (ids: string[]) => void;
};

/** The seams on a row of the strip's height, the cuts closed up, putting
 * them back as the editor would; Controls start it afresh. */
function LiveSeams({ cuts: initial, onRestore }: StoryProps) {
  const [cuts, setCuts] = useState(initial);
  const scale = timelineScale(
    squeezedTime(
      DURATION,
      cuts.map((cut) => ({ ...cut, plays: 0 })),
    ),
  );
  return (
    <div className="relative mt-2 h-11" dir="ltr" style={{ width: WIDTH }}>
      <CutSeams
        cuts={cuts}
        onRestore={(ids) => {
          onRestore(ids);
          setCuts(cuts.filter((cut) => !ids.includes(cut.id)));
        }}
        scale={scale}
      />
    </div>
  );
}

const meta = {
  args: {
    // One cut on its own, and two that touch, which close up as one seam.
    cuts: [
      { end: 20, id: "first", start: 12 },
      { end: 36, id: "second", start: 30 },
      { end: 41, id: "third", start: 36 },
    ],
    onRestore: fn(),
  },
  component: LiveSeams,
  decorators: [featureStage({ width: WIDTH })],
  parameters: { controls: { include: ["cuts"] } },
  render: (args) => <LiveSeams key={JSON.stringify(args.cuts)} {...args} />,
  title: "Features/Editor/Timeline/Cut Seams",
} satisfies Meta<typeof LiveSeams>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
