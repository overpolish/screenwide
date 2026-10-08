// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingTimelineEdit,
  setRecordingTimelineSegmentPlaybackRate,
} from "../editing/recording-timeline-edit";

import { RecordingTrackLanesPreview } from "./recording-track-lanes-preview";
import { STORY_CUT_EDIT } from "./recording-track-lanes-preview-fixtures";

import type { Meta, StoryObj } from "@storybook/react-vite";

function TimelinePreview({
  initialEdit,
}: {
  initialEdit?: RecordingTimelineEdit;
}) {
  return (
    <div className="w-[760px]">
      <RecordingTrackLanesPreview initialEdit={initialEdit} />
    </div>
  );
}

const meta = {
  component: TimelinePreview,
  parameters: { layout: "centered" },
  title: "Features/Editor/Timeline",
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
