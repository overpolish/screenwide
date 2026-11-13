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

const MANY_KINDS = [
  "arrow",
  "highlight",
  "counter",
  "text",
  "focus",
  "redact",
  "spotlight",
  "magnify",
  "shape",
] as const;

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
    isAnnotationsHidden: false,
    isCaptionsHidden: false,
    isCutting: false,
    isPlaying: false,
    isRanging: false,
    isScreenShown: true,
    isShortcutsHidden: false,
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
        "isAnnotationsHidden",
        "isCaptionsHidden",
        "isCutting",
        "isPlaying",
        "isRanging",
        "isScreenShown",
        "isShortcutsHidden",
        "squeeze",
      ],
    },
  },
  title: "Features/Editor/Timeline/Strip",
} satisfies Meta<typeof Live>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Fits whole as it opens, nothing squeezed: the squeeze's handle is at the
// end.
export const Short: Story = {
  args: { recording: storyRecording({ duration: 24 }), squeeze: 0 },
};

// What the squeeze took is hatched past the end, the handle where the hatch
// begins.
export const Squeezed: Story = {
  args: { recording: storyRecording({ duration: 24 }), squeeze: 1 },
};

// A mark at the foot of the beats wherever play jumps, which puts the move
// back when pressed.
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

// Two songs end to end on their rail, the second looping and chosen, and
// two sound effects that overlap, each in a lane of its own.
export const SoundTrack: Story = {
  args: {
    audio: {
      clips: [
        {
          end: 40,
          id: "intro",
          isLooped: false,
          kind: "music",
          lane: 0,
          start: 0,
          title: "Habanera from Carmen (Georges Bizet) - Golden Hour Strings",
        },
        {
          end: 90,
          id: "theme",
          isLooped: true,
          kind: "music",
          lane: 0,
          start: 40,
          title: "Long Theme",
        },
        {
          end: 24,
          id: "whoosh",
          isLooped: false,
          kind: "effect",
          lane: 0,
          start: 12,
          title: "Whoosh",
        },
        {
          end: 15,
          id: "click",
          isLooped: false,
          kind: "effect",
          lane: 1,
          start: 13,
          title: "Click",
        },
      ],
      onRestack: fn(),
      onRetime: fn(),
    },
    selection: { id: "theme", kind: "audioClip" },
  },
};

// A stretch drawn across a lull and into the beat after it.
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

export const Cuts: Story = {
  args: { recording: storyRecording({ duration: 90, isCut: true }) },
};

// The band past the end holds what was squeezed, then what was deleted;
// restoring a cut or squeezing glides the strip and band together.
export const CutAndSqueezed: Story = {
  args: { recording: storyRecording({ duration: 60, isCut: true }) },
};

export const CrowdedAnnotations: Story = {
  args: { recording: storyRecording({ duration: 90, isCrowded: true }) },
};

// The lanes grow to four and scroll past them; the rest stays put.
export const ManyAnnotations: Story = {
  args: {
    recording: {
      ...recording,
      annotations: Array.from({ length: 20 }, (_, index) => {
        const start = 20 + (index % 5);
        return {
          end: start + 12,
          id: `annotation-${String(index)}`,
          kind: MANY_KINDS[index % MANY_KINDS.length],
          lane: index,
          start,
        };
      }),
    },
  },
};

// Nothing said, so there is no caption lane.
export const NoVoice: Story = {
  args: { recording: storyRecording({ duration: 90, isVoiced: false }) },
};

export const ThumbnailsLoading: Story = {
  args: { recording: storyRecording({ duration: 90, isLoading: true }) },
};

// The view follows the playhead, a page at a time.
export const Playing: Story = { args: { isPlaying: true } };

// A blade follows the pointer, a drag draws a stretch to cut, and the chosen
// lull's end trims.
export const OptionHeld: Story = {
  args: {
    isAltHeld: true,
    selection: longLull ? { id: longLull.id, kind: "lull" } : null,
  },
};
