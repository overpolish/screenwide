// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { Text } from "../../../../components/base/text/text";
import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";
import { Annotation } from "../../annotations/annotations";
import { formatDuration } from "../../duration";
import { createRecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { fitTimelineViewport } from "../../timeline/timeline-viewport";

import { RecordingAnnotationLane } from "./recording-annotation-lane";
import {
  clearedPinCorrections,
  pinnedClip,
  unpinnedClip,
} from "./recording-annotation-pins";
import { RecordingAnnotationClip } from "./recording-annotations";
import { AnnotationClipPinning } from "./use-annotation-clip-menu";
import { useLaneAnnotationSelection } from "./use-lane-annotation-selection";

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
      blur: false,
      color: "#ff383c",
      handDrawn: false,
      head: "end",
      manual: false,
      radius: 0,
      redaction: "erase",
      shadow: false,
      softness: 0,
      strength: 0,
      tint: false,
      width: 12,
    },
  } satisfies Annotation,
  endMs: end,
  startMs: start,
  trackId: "primary",
});

/** Pinning as the editor does it, pinned where each annotation was placed
 * since a story has no playhead in source time, and with no tracker to bring
 * a hidden annotation back from. */
const pinning = (
  setClips: (
    change: (clips: RecordingAnnotationClip[]) => RecordingAnnotationClip[],
  ) => void,
): AnnotationClipPinning => {
  const withClip =
    (change: (clip: RecordingAnnotationClip) => RecordingAnnotationClip) =>
    (id: string) => {
      setClips((clips) =>
        clips.map((clip) => (clip.annotation.id === id ? change(clip) : clip)),
      );
    };
  return {
    canHideHere: () => false,
    canShowHere: () => false,
    onClearCorrections: withClip((clip) =>
      clip.pin ? { ...clip, pin: clearedPinCorrections(clip.pin) } : clip,
    ),
    onHideHere: () => undefined,
    onPinnedChange: (id, pinned) => {
      withClip((clip) => (pinned ? pinnedClip(clip, -1) : unpinnedClip(clip)))(
        id,
      );
    },
    onShowHere: () => undefined,
  };
};

function Preview() {
  const [clips, setClips] = useState(() => [
    arrow("arrow-a", 8_000, 31_000),
    arrow("arrow-b", 22_000, 48_000),
  ]);
  const [position, setPosition] = useState(0.1);
  const selection = useLaneAnnotationSelection(clips);
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
        onSeek={setPosition}
        pinning={pinning(setClips)}
        {...selection}
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

/** Two of three arrows chosen together: carrying either one moves both in
 * time and through the drawing order, keeping their spacing. */
export const Group: Story = {
  render: function GroupLane() {
    const [clips, setClips] = useState(() => [
      arrow("arrow-a", 8_000, 31_000),
      arrow("arrow-b", 22_000, 48_000),
      arrow("arrow-c", 40_000, 70_000),
    ]);
    const selection = useLaneAnnotationSelection(clips, ["arrow-a", "arrow-c"]);
    return (
      <div className="p-window-inset">
        <RecordingAnnotationLane
          clips={clips}
          edit={createRecordingTimelineEdit(1)}
          onChange={setClips}
          sourceDurationMs={120_000}
          viewport={fitTimelineViewport()}
          {...selection}
        />
      </div>
    );
  },
};

export const Empty: Story = {
  render: () => (
    <div className="p-window-inset">
      <RecordingAnnotationLane
        clips={[]}
        edit={createRecordingTimelineEdit(1)}
        onChange={() => undefined}
        onClearSelection={() => undefined}
        onSelect={() => undefined}
        onSelectSwept={() => undefined}
        selectedIds={new Set()}
        sourceDurationMs={120_000}
        viewport={fitTimelineViewport()}
      />
    </div>
  ),
};

/** Two pinned arrows, each showing its keyframes and stretches whether in
 * hand or not. The one in hand was put right once, followed poorly in one
 * stretch and off the frame in another; the other is still being tracked,
 * and its content was said to be out of view for ten seconds. */
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
        pin: {
          keyframes: [
            { dx: 0, dy: 0, ms: 60_000 },
            { dx: 0, dy: 0, ms: 70_000, outOfView: true },
            { dx: 4, dy: -12, ms: 80_000 },
          ],
          pinnedMs: 60_000,
        },
      },
    ]);
    const selection = useLaneAnnotationSelection(clips, ["arrow-a"]);
    return (
      <div className="p-window-inset">
        <RecordingAnnotationLane
          clips={clips}
          edit={createRecordingTimelineEdit(1)}
          onChange={setClips}
          pinning={pinning(setClips)}
          pinStatus={
            new Map([
              [
                "arrow-a",
                {
                  covered: [[24_000, 27_000]],
                  hidden: [[30_000, 34_000]],
                  progress: null,
                  under: [[24_000, 27_000]],
                  weak: [[17_000, 21_000]],
                },
              ],
              [
                "arrow-b",
                { covered: [], hidden: [], progress: 0.4, under: [], weak: [] },
              ],
            ])
          }
          {...selection}
          sourceDurationMs={120_000}
          viewport={fitTimelineViewport()}
        />
      </div>
    );
  },
};
