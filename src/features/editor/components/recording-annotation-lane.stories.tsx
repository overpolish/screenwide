// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { Text } from "../../../components/base/text/text";
import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { Annotation } from "../annotations";
import { formatDuration } from "../duration";
import {
  clearedPinCorrections,
  pinnedClip,
  unpinnedClip,
} from "../recording-annotation-pins";
import { RecordingAnnotationClip } from "../recording-annotations";
import { createRecordingTimelineEdit } from "../recording-timeline-edit";

import { RecordingAnnotationLane } from "./recording-annotation-lane";
import { fitTimelineViewport } from "./timeline-viewport";

import type { Meta, StoryObj } from "@storybook/react-vite";

const arrow = (
  id: string,
  start: number,
  end: number,
): RecordingAnnotationClip => ({
  annotation: {
    aboveCamera: false,
    animated: true,
    id,
    shape: {
      control: { x: 320, y: 80 },
      end: { x: 600, y: 180 },
      kind: "arrow",
      start: { x: 40, y: 40 },
    },
    style: {
      align: "left",
      color: "#ff383c",
      head: "end",
      radius: 0,
      redaction: "erase",
      strength: 0,
      width: 12,
    },
  } satisfies Annotation,
  endMs: end,
  startMs: start,
  trackId: "primary",
});

/** Pinning as the editor does it, pinned where each annotation was placed
 * since a story has no playhead in source time. */
const pinning = (
  setClips: (
    change: (clips: RecordingAnnotationClip[]) => RecordingAnnotationClip[],
  ) => void,
) => {
  const withClip =
    (change: (clip: RecordingAnnotationClip) => RecordingAnnotationClip) =>
    (id: string) => {
      setClips((clips) =>
        clips.map((clip) => (clip.annotation.id === id ? change(clip) : clip)),
      );
    };
  return {
    onClearPinCorrections: withClip((clip) =>
      clip.pin ? { ...clip, pin: clearedPinCorrections(clip.pin) } : clip,
    ),
    onPinnedChange: (id: string, pinned: boolean) => {
      withClip((clip) => (pinned ? pinnedClip(clip, -1) : unpinnedClip(clip)))(
        id,
      );
    },
  };
};

function Preview() {
  const [clips, setClips] = useState(() => [
    arrow("arrow-a", 8_000, 31_000),
    arrow("arrow-b", 22_000, 48_000),
  ]);
  const [position, setPosition] = useState(0.1);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  return (
    <div className="flex flex-col gap-section p-window-inset">
      <Text aria-label="Playhead" className="font-mono" variant="footnote">
        {formatDuration(position * 66000)}
      </Text>
      <RecordingAnnotationLane
        clips={clips}
        edit={{
          ...createRecordingTimelineEdit(1),
          segments: [
            { id: 0, sourceEnd: 0.2, sourceStart: 0 },
            { id: 1, playbackRate: 2, sourceEnd: 1, sourceStart: 0.3 },
          ],
        }}
        onChange={setClips}
        {...pinning(setClips)}
        onSeek={setPosition}
        onSelect={setSelectedId}
        selectedId={selectedId}
        sourceDurationMs={120_000}
        viewport={fitTimelineViewport()}
      />
    </div>
  );
}

const meta = {
  component: Preview,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={760}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "centered" },
  title: "Features/Editor/Recording Annotation Lane",
} satisfies Meta<typeof Preview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Empty: Story = {
  render: () => (
    <div className="p-window-inset">
      <RecordingAnnotationLane
        clips={[]}
        edit={createRecordingTimelineEdit(1)}
        onChange={() => undefined}
        onSelect={() => undefined}
        selectedId={null}
        sourceDurationMs={120_000}
        viewport={fitTimelineViewport()}
      />
    </div>
  ),
};

/** Two pinned arrows. The one in hand shows where it was pinned and put
 * right, a stretch it was followed poorly in to check, and a stretch its
 * content was off the frame; the other is still being tracked. */
export const Pinned: Story = {
  render: function PinnedLane() {
    const [clips, setClips] = useState<RecordingAnnotationClip[]>(() => [
      {
        ...arrow("arrow-a", 4_000, 40_000),
        pin: {
          keyframes: [
            { dx: 0, dy: 0, ms: 10_000 },
            { dx: 12, dy: -30, ms: 26_000 },
          ],
          pinnedMs: 10_000,
        },
      },
      {
        ...arrow("arrow-b", 50_000, 90_000),
        pin: { keyframes: [{ dx: 0, dy: 0, ms: 60_000 }], pinnedMs: 60_000 },
      },
    ]);
    const [selectedId, setSelectedId] = useState<string | null>("arrow-a");
    return (
      <div className="p-window-inset">
        <RecordingAnnotationLane
          clips={clips}
          edit={createRecordingTimelineEdit(1)}
          onChange={setClips}
          {...pinning(setClips)}
          onSelect={setSelectedId}
          pinStatus={
            new Map([
              [
                "arrow-a",
                {
                  hidden: [[30_000, 34_000]],
                  progress: null,
                  weak: [[17_000, 21_000]],
                },
              ],
              ["arrow-b", { hidden: [], progress: 0.4, weak: [] }],
            ])
          }
          selectedId={selectedId}
          sourceDurationMs={120_000}
          viewport={fitTimelineViewport()}
        />
      </div>
    );
  },
};
