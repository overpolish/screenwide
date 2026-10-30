// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";

import { storyRecording } from "./strip-story-fixtures";
import { LiveStrip, type LiveStripProps } from "./strip-story-live";

import type { Meta, StoryObj } from "@storybook/react-vite";

// The editor window's width, which the strip runs across.
const stage = featureStage({ width: 1200 });

const recording = storyRecording({ duration: 90 });
const longLull = recording.lulls.find(
  ({ moving, span }) => span.end - span.start > 6 && !moving,
);

/** The strip as Controls set it; they start it afresh. */
const Live = (props: LiveStripProps) => (
  <LiveStrip
    key={JSON.stringify([props.selection, props.squeeze, props.recording])}
    {...props}
  />
);

const meta = {
  argTypes: {
    squeeze: { control: { max: 1, min: 0, step: 0.05, type: "range" } },
  },
  args: {
    isAltHeld: false,
    isCaptionsHidden: false,
    isCutting: false,
    isPlaying: false,
    moves: [],
    onAnnotationRestack: fn(),
    onAnnotationRetime: fn(),
    onBeatTrim: fn(),
    onCaptionPress: fn(),
    onDeleteShortcuts: fn(),
    onLullPlayChange: fn(),
    onLullTrim: fn(),
    onMove: fn(),
    onPutBack: fn(),
    onRemoveSplit: fn(),
    onRestore: fn(),
    onSceneDraw: fn(),
    onSceneRetime: fn(),
    onSeek: fn(),
    onSelect: fn(),
    onSplit: fn(),
    onSqueezeChange: fn(),
    recording,
    selection: null,
    squeeze: 0.5,
  },
  component: Live,
  decorators: [stage],
  parameters: {
    controls: {
      include: [
        "isAltHeld",
        "isCaptionsHidden",
        "isCutting",
        "isPlaying",
        "squeeze",
      ],
    },
  },
  title: "Features/Editor/Timeline/Strip",
} satisfies Meta<typeof Live>;

export default meta;
type Story = StoryObj<typeof meta>;

// A minute and a half, opened on its first minute or so.
export const Default: Story = {};

// Short enough to fit whole as it opens.
export const Short: Story = {
  args: { recording: storyRecording({ duration: 24 }) },
};

// The fourth beat moved to play after the ninth: a mark at the foot of the
// beats wherever play jumps, each putting it back when pressed.
export const Reordered: Story = {
  args: {
    moves: [
      {
        before: recording.beats[9].span.start,
        span: recording.beats[3].span,
      },
    ],
  },
};

// Eight minutes: the minimap's window is a sliver of it.
export const Long: Story = {
  args: { recording: storyRecording({ duration: 480 }) },
};

export const BeatSelected: Story = {
  args: { selection: { id: "beat-4", kind: "beat" } },
};

export const LullSelected: Story = {
  args: {
    selection: longLull ? { id: longLull.id, kind: "lull" } : null,
  },
};

export const SceneSelected: Story = {
  args: { selection: { id: "demo", kind: "scene" } },
};

export const AnnotationSelected: Story = {
  args: { selection: { id: "text", kind: "annotation" } },
};

// A stretch the scissors drew across a lull and into the beat after it.
export const RangeSelected: Story = {
  args: {
    selection: longLull
      ? {
          end: longLull.span.end + 1,
          kind: "range",
          start: (longLull.span.start + longLull.span.end) / 2,
        }
      : null,
  },
};

// Two beats with their middles cut, each leaving a seam to put it back.
export const Cuts: Story = {
  args: { recording: storyRecording({ duration: 90, isCut: true }) },
};

export const CrowdedAnnotations: Story = {
  args: { recording: storyRecording({ duration: 90, isCrowded: true }) },
};

// Nothing said, so there is no caption lane.
export const NoVoice: Story = {
  args: { recording: storyRecording({ duration: 90, isVoiced: false }) },
};

// The pictures not read from the video yet.
export const ThumbnailsLoading: Story = {
  args: { recording: storyRecording({ duration: 90, isLoading: true }) },
};

// The view follows the playhead, a page at a time.
export const Playing: Story = { args: { isPlaying: true } };

// With ⌥ held the scissors are in hand: a blade follows the pointer, a drag
// draws a stretch to cut, and the chosen lull's end trims.
export const OptionHeld: Story = {
  args: {
    isAltHeld: true,
    selection: longLull ? { id: longLull.id, kind: "lull" } : null,
  },
};
