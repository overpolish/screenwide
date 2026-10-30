// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../../storybook/story-grid";
import { recordedTime, timelineScale } from "../timeline-scale";

import { CaptionLine, type CaptionLineProps } from "./caption-line";
import { storyCaption } from "./caption-story-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** How wide a second is drawn, in px. */
const PX_PER_S = 10;

/** One line alone, filling a strip as long as it is, so a short one in a
 * grid leaves no room empty beside it. */
function Line(props: Omit<CaptionLineProps, "scale" | "width">) {
  const width = props.line.end * PX_PER_S;
  return (
    <div className="relative h-4" dir="ltr" style={{ width }}>
      <CaptionLine
        {...props}
        scale={timelineScale(recordedTime(props.line.end))}
        width={width}
      />
    </div>
  );
}

// Room to spread its words, to show its text cut short, and for nothing.
const WIDTHS = [
  { label: "Spread", line: storyCaption(0, 20, "Open the app") },
  {
    label: "Cut short",
    line: storyCaption(0, 12, "Pick a template, then name it so we find it"),
  },
  { label: "Bare", line: storyCaption(0, 5, "Done") },
];

const meta = {
  args: {
    isHidden: false,
    line: WIDTHS[0].line,
    onPress: fn(),
    // Each story draws its line on a strip as long as it is.
    scale: timelineScale(recordedTime(WIDTHS[0].line.end)),
    width: WIDTHS[0].line.end * PX_PER_S,
  },
  component: CaptionLine,
  parameters: { controls: { include: ["isHidden", "line"] } },
  render: (args) => <Line {...args} />,
  title: "Features/Editor/Timeline/Caption Line",
} satisfies Meta<typeof CaptionLine>;

export default meta;
type Story = StoryObj<typeof meta>;

// Each story's stage is as wide as what it draws, so it sits centred and
// never scrolls for room it does not use.
export const Default: Story = {
  decorators: [featureStage({ width: WIDTHS[0].line.end * PX_PER_S })],
};

// Hidden from the picture, a line is an outline.
export const States: Story = {
  decorators: [featureStage({ width: 448 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={WIDTHS.map(({ label }) => label)}>
      {[
        { isHidden: false, label: "Shown" },
        { isHidden: true, label: "Hidden" },
      ].map(({ label, ...state }) => (
        <StoryGridRow key={label} label={label}>
          {WIDTHS.map(({ label: width, line }) => (
            <Line key={width} {...args} {...state} line={line} />
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
