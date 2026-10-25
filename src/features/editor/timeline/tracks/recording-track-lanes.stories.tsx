// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import {
  RecordingTimelineEdit,
  setRecordingTimelineSegmentPlaybackRate,
} from "../editing/recording-timeline-edit";
import { moveRecordingTimelineSegment } from "../editing/recording-timeline-move";

import { RecordingTrackLanesPreview } from "./recording-track-lanes-preview";
import {
  STORY_CAMERA_BACKGROUND_PENDING,
  STORY_CAMERA_BACKGROUND_RANGES,
  STORY_CAMERA_BACKGROUND_WORKING,
  STORY_CUT_EDIT,
} from "./recording-track-lanes-preview-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

function TimelinePreview({
  camera,
  initialEdit,
}: {
  camera?: ComponentProps<typeof RecordingTrackLanesPreview>["camera"];
  initialEdit?: RecordingTimelineEdit;
}) {
  return (
    <div className="w-[760px]">
      <RecordingTrackLanesPreview camera={camera} initialEdit={initialEdit} />
    </div>
  );
}

const meta = {
  component: TimelinePreview,
  parameters: { layout: "centered" },
  title: "Legacy/Editor/Timeline/Tracks",
} satisfies Meta<typeof TimelinePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ZoomAndPan: Story = {};

/** Pauses cut out of the recording, each marked over the ruler where its
 * segments meet. A marker puts its pause back and joins the segments again. */
export const Cuts: Story = {
  args: { initialEdit: STORY_CUT_EDIT },
};

/** Cuts beside clips played at another rate: the second clip at 2×, between
 * two cuts, and the last at 1.5×, so the retime strips and the cut markers
 * share the ruler's lower half. Restoring a cut next to a retimed clip plays
 * the restored pause at the rate of the clip before it. */
export const CutsWithSpeed: Story = {
  args: {
    initialEdit: setRecordingTimelineSegmentPlaybackRate(
      setRecordingTimelineSegmentPlaybackRate(STORY_CUT_EDIT, 1, 2),
      3,
      1.5,
    ),
  },
};

/** The last clip moved to the start, so the clips no longer play in the
 * order they were recorded. Restore original order, beside Snap, puts them
 * back once pressed twice; undo moves them out again. */
export const Reordered: Story = {
  args: {
    initialEdit: moveRecordingTimelineSegment(STORY_CUT_EDIT, {
      boundary: 0,
      segmentId: STORY_CUT_EDIT.segments[STORY_CUT_EDIT.segments.length - 1].id,
    }),
  },
};

/** The recording trimmed at both ends: each end has a marker lying wholly
 * inside the timeline, which reaches the clip beside it back to that end. The
 * first clip plays at 2×, so its retime strip clears the marker at the start. */
export const TrimmedEnds: Story = {
  args: {
    initialEdit: {
      artifactId: 1,
      nextSegmentId: 2,
      segments: [
        { id: 0, playbackRate: 2, sourceEnd: 0.5, sourceStart: 0.04 },
        { id: 1, sourceEnd: 0.93, sourceStart: 0.5 },
      ],
    },
  },
};

/** The camera's background taken away in three stretches, each with its own
 * background. A range slides along the strip over the camera and its edges
 * trim it; Delete takes the chosen ones away, and the strip closes once the
 * last is gone. */
export const CameraBackgroundRanges: Story = {
  args: { camera: { ranges: STORY_CAMERA_BACKGROUND_RANGES } },
};

/** The background taken away over the whole recording, worked out from the
 * playhead outward: the stretches still to come are faded. */
export const CameraBackgroundWorking: Story = {
  args: {
    camera: {
      pending: STORY_CAMERA_BACKGROUND_PENDING,
      ranges: STORY_CAMERA_BACKGROUND_WORKING,
    },
  },
};
