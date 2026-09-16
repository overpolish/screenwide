// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { Text } from "../../../components/base/text/text";
import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { Annotation } from "../annotations";
import { formatDuration } from "../duration";
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
    style: { color: "#ff383c", head: "end", width: 12 },
  } satisfies Annotation,
  endMs: end,
  startMs: start,
  trackId: "primary",
});

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
