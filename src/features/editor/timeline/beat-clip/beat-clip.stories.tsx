// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ANNOTATION_SWATCHES } from "../../../../composites/color-grid/palette";
import { featureStage } from "../../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../../storybook/story-grid";
import { recordedTime, timelineScale } from "../timeline-scale";
import {
  storyFrame,
  storyFrameAspect,
  storyPeaks,
} from "../timeline-story-fixtures";

import { BeatClip, type BeatClipProps } from "./beat-clip";
import { type TimelineBeat } from "./beat-label";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A strip 24 seconds long, 240 px wide, so a second is 10 px.
const WIDTH = 240;
const scale = timelineScale(recordedTime(24));

/** One beat, at the start of a row of the strip's height. */
function Row(props: BeatClipProps) {
  return (
    <div className="relative h-11" dir="ltr" style={{ width: WIDTH }}>
      <BeatClip {...props} />
    </div>
  );
}

/** One beat alone, filling a strip as long as it is, so a short one in a
 * grid leaves no room empty beside it. */
function Fitted({
  seconds,
  ...props
}: Omit<BeatClipProps, "scale" | "span" | "width"> & { seconds: number }) {
  const width = seconds * 10;
  return (
    <div className="relative h-11" dir="ltr" style={{ width }}>
      <BeatClip
        {...props}
        scale={timelineScale(recordedTime(seconds))}
        span={{ end: seconds, start: 0 }}
        width={width}
      />
    </div>
  );
}

const KINDS: { beat: TimelineBeat; label: string }[] = [
  { beat: { kind: "click", label: "New Project" }, label: "Click" },
  { beat: { kind: "type", label: "Onboarding" }, label: "Type" },
  { beat: { keys: ["Command", "S"], kind: "keys" }, label: "Keys" },
  { beat: { app: "Safari", kind: "switch" }, label: "Switch" },
  { beat: { kind: "screen" }, label: "New screen" },
  { beat: { kind: "sound" }, label: "Loud moment" },
  {
    beat: { kind: "speech", line: "Let's set up a project from a template." },
    label: "Speech",
  },
  {
    beat: { color: "#ff8d28", kind: "moment", name: "Notable" },
    label: "Moment",
  },
];

const meta = {
  argTypes: {
    isHidden: { table: { defaultValue: { summary: "false" } } },
  },
  args: {
    beat: { kind: "click", label: "New Project" },
    isHidden: false,
    isSelected: false,
    onPress: fn(),
    peaks: storyPeaks(1),
    pictureAspect: storyFrameAspect(),
    scale,
    span: { end: 23, start: 0 },
    thumbnail: storyFrame(0),
    width: WIDTH,
  },
  component: BeatClip,
  parameters: {
    controls: { include: ["beat", "isHidden", "isSelected", "span"] },
  },
  render: (args) => <Row {...args} />,
  title: "Features/Editor/Timeline/Beat Clip",
} satisfies Meta<typeof BeatClip>;

export default meta;
type Story = StoryObj<typeof meta>;

// Each story's stage is as wide as what it draws, so it sits centred and
// never scrolls for room it does not use.
export const Default: Story = {
  decorators: [featureStage({ width: WIDTH })],
};

export const Kinds: Story = {
  decorators: [featureStage({ width: 336 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {KINDS.map(({ beat, label }, index) => (
        <StoryGridRow key={label} label={label}>
          <Row
            {...args}
            beat={beat}
            peaks={storyPeaks(index)}
            thumbnail={storyFrame(index)}
          />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

// What a beat shows grows with its width: bare, its glyph, its picture,
// then its title.
export const Widths: Story = {
  decorators: [featureStage({ width: 296 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {[
        { label: "Bare", seconds: 1 },
        { label: "Glyph", seconds: 3 },
        { label: "Picture", seconds: 9 },
        { label: "Title", seconds: 23 },
      ].map(({ label, seconds }) => (
        <StoryGridRow key={label} label={label}>
          <Row {...args} span={{ end: seconds, start: 0 }} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

// At rest, chosen, and with what it draws over the picture hidden.
export const States: Story = {
  decorators: [featureStage({ width: 808 })],
  parameters: { controls: { disable: true } },
  render: (args) => {
    const beats: TimelineBeat[] = [
      { kind: "click", label: "New Project" },
      { kind: "speech", line: "Let's set up a project from a template." },
      { color: "#ff8d28", kind: "moment", name: "Notable" },
    ];
    return (
      <StoryGrid columns={["Action", "Speech", "Moment"]}>
        {[
          { isHidden: false, isSelected: false, label: "Rest" },
          { isHidden: false, isSelected: true, label: "Selected" },
          { isHidden: true, isSelected: false, label: "Hidden" },
        ].map(({ label, ...state }) => (
          <StoryGridRow key={label} label={label}>
            {beats.map((beat) => (
              <Row key={beat.kind} {...args} {...state} beat={beat} />
            ))}
          </StoryGridRow>
        ))}
      </StoryGrid>
    );
  },
};

// A moment in each colour a kind can take: its flag in the colour, ink
// that reads on it, and too narrow for the flag, the colour alone.
export const MomentColors: Story = {
  decorators: [featureStage({ width: 184 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["Flag", "Bare"]}>
      {ANNOTATION_SWATCHES.map(({ color, name }) => (
        <StoryGridRow key={color} label={name()}>
          {[9, 1].map((seconds) => (
            <Fitted
              key={seconds}
              {...args}
              beat={{ color, kind: "moment", name: name() }}
              seconds={seconds}
            />
          ))}
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

// A recording's own shape, wide, square or tall, kept by its thumbnail;
// only its width changes, so the row of glyphs and titles keeps its height.
export const ThumbnailShapes: Story = {
  decorators: [featureStage({ width: 648 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["Picture", "Title", "Moment"]}>
      {(["wide", "square", "tall"] as const).map((shape, index) => (
        <StoryGridRow key={shape} label={shape}>
          <Fitted
            {...args}
            pictureAspect={storyFrameAspect(shape)}
            seconds={9}
            thumbnail={storyFrame(index, shape)}
          />
          <Row
            {...args}
            pictureAspect={storyFrameAspect(shape)}
            thumbnail={storyFrame(index, shape)}
          />
          <Row
            {...args}
            beat={{ color: "#ff8d28", kind: "moment", name: "Notable" }}
            pictureAspect={storyFrameAspect(shape)}
            thumbnail={storyFrame(index, shape)}
          />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};

// The pictures are still being read from the video: each place is held at
// the recording's shape, so nothing moves when they arrive.
export const Loading: Story = {
  decorators: [featureStage({ width: 528 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid columns={["Action", "Moment"]}>
      {(["wide", "square", "tall"] as const).map((shape) => (
        <StoryGridRow key={shape} label={shape}>
          <Row
            {...args}
            pictureAspect={storyFrameAspect(shape)}
            thumbnail={null}
          />
          <Row
            {...args}
            beat={{ color: "#ff8d28", kind: "moment", name: "Notable" }}
            pictureAspect={storyFrameAspect(shape)}
            thumbnail={null}
          />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
