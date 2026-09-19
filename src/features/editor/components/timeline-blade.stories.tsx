// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import {
  createRecordingTimelineEdit,
  cutRecordingTimeline,
  RecordingTimelineEdit,
  setRecordingTimelineSegmentPlaybackRate,
} from "../recording-timeline-edit";
import { RecordingTimelineThumbnail } from "../types";

import { TimelineBladeController } from "./timeline-blade";
import { TimelineVideoClip } from "./timeline-video-clip";
import { fitTimelineViewport } from "./timeline-viewport";

import type { Meta, StoryObj } from "@storybook/react-vite";

const CUT_EDIT = cutRecordingTimeline(
  cutRecordingTimeline(createRecordingTimelineEdit(1), 0.35),
  0.7,
);
const SPED_UP_EDIT = setRecordingTimelineSegmentPlaybackRate(CUT_EDIT, 1, 2);

const thumbnails: RecordingTimelineThumbnail[] = Array.from(
  { length: 24 },
  (_, index) => ({ id: `frame-${index.toString()}`, url: null }),
);

/**
 * The segments of an edit as one lane draws them, with the real lane driven
 * by a controller held in story state. The blade tool itself is one layer
 * over the whole lanes column rather than a part of a lane, so it belongs to
 * the timeline story; trim and speed edits that need the native cursor or a
 * menu are inert here.
 */
function BladePreview({
  initialEdit = CUT_EDIT,
  selectedSegmentId = null,
}: {
  initialEdit?: RecordingTimelineEdit;
  selectedSegmentId?: number | null;
}) {
  const [edit, setEdit] = useState(initialEdit);
  const [selected, setSelected] = useState(selectedSegmentId);
  const [preview, setPreview] = useState<number | null>(null);
  const blade: TimelineBladeController = {
    beginTrim: () => undefined,
    clearPreview: () => {
      setPreview(null);
    },
    clearRangeSelection: () => undefined,
    cutAt: (sourcePosition) => {
      setEdit((current) => cutRecordingTimeline(current, sourcePosition));
    },
    edit,
    endTrim: () => undefined,
    isActive: false,
    isRangeActive: false,
    isSnapActive: false,
    previewAt: setPreview,
    previewPosition: preview,
    rangeSelection: null,
    selectSegment: setSelected,
    selectedSegmentId: selected,
    setActive: () => undefined,
    setRangeActive: () => undefined,
    setRangePlaybackRate: () => undefined,
    setRangeSelection: () => undefined,
    setSegmentPlaybackRate: (segmentId, playbackRate) => {
      setEdit((current) =>
        setRecordingTimelineSegmentPlaybackRate(
          current,
          segmentId,
          playbackRate,
        ),
      );
    },
    setSnapActive: () => undefined,
    setSnapGuidePosition: () => undefined,
    snapGuidePosition: null,
    snapPosition: (sourcePosition) => sourcePosition,
    updateTrim: () => null,
  };

  return (
    <div className="flex w-[760px]">
      <TimelineVideoClip
        blade={blade}
        enabled
        onSelect={() => undefined}
        selected={selected !== null}
        thumbnails={thumbnails}
        trackId="primary"
        viewport={fitTimelineViewport()}
      />
    </div>
  );
}

const meta = {
  component: BladePreview,
  parameters: { layout: "centered" },
  title: "Features/Editor/Timeline Blade",
} satisfies Meta<typeof BladePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Three segments with their trim handles, on the lane's fill. */
export const Segments: Story = {};

/** A single uncut clip: one segment filling the lane. */
export const SingleSegment: Story = {
  args: { initialEdit: createRecordingTimelineEdit(1) },
};

/** A picked segment, tinted with the accent the app selects with. */
export const SelectedSegment: Story = {
  args: { selectedSegmentId: 1 },
};

/** A segment playing back at a different rate carries its speed badge. */
export const SegmentSpeed: Story = {
  args: { initialEdit: SPED_UP_EDIT },
};
