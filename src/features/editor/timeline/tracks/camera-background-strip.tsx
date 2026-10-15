// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  CAMERA_BACKGROUND_RANGE_MINIMUM_MS,
  CameraBackgroundRange,
  cameraBackgroundRangeLabel,
} from "../../recording/camera/camera-background-ranges";
import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { SeekHandler } from "../timeline-seek";
import { TimelineViewportState } from "../timeline-viewport";

import { TIMED_LANE_ROW_HEIGHT_PX } from "./timed-lane-layout";
import { TimedSpan, TimedSpanStrip } from "./timed-span-strip";

/** The camera's background ranges, and what the timeline does with them. */
export type CameraBackgroundLane = {
  onChange: (ranges: CameraBackgroundRange[]) => void;
  /** Let every range go, from a click on empty strip. */
  onClearSelection: () => void;
  /** Choose the range `id` alone, or with `toggle`, add or take it away. */
  onSelect: (id: string, toggle: boolean) => void;
  /** Choose the ranges a band swept over, alone or added to the choice. */
  onSelectSwept: (ids: string[], additive: boolean) => void;
  ranges: CameraBackgroundRange[];
  selectedIds: ReadonlySet<string>;
  /** A range was clicked rather than dragged: show the camera's panel. */
  onActivate?: () => void;
  /** Shows a drag's draft in the preview, or the committed ranges for null. */
  onDraftChange?: (ranges: CameraBackgroundRange[] | null) => void;
  /** The parts of `range` whose background is still being worked out, in
   * milliseconds into the edited video. */
  pendingSpans?: (range: CameraBackgroundRange) => readonly TimedSpan[];
};

/**
 * The camera's background ranges as boxes over its clip, inside the fill the
 * camera's row puts round both. The strip opens only while the camera has a
 * range, so a camera without one keeps the height of every other track. A
 * camera left out of the export dims its ranges with it.
 */
export function CameraBackgroundStrip({
  edit,
  isEnabled,
  lane,
  onSeek,
  onSelectTrack,
  sourceDurationMs,
  viewport,
}: {
  edit: RecordingTimelineEdit;
  isEnabled: boolean;
  lane: CameraBackgroundLane;
  /** Choose the camera track, as a press anywhere on its row does. */
  onSelectTrack: () => void;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  onSeek?: SeekHandler;
}) {
  const isOpen = lane.ranges.length > 0;
  return (
    <div
      aria-hidden={!isOpen}
      className="flex shrink-0 overflow-hidden transition-[height] duration-200"
      style={{ height: isOpen ? TIMED_LANE_ROW_HEIGHT_PX : 0 }}
    >
      <TimedSpanStrip
        clips={lane.ranges}
        edit={edit}
        isIdle={() => !isEnabled}
        label={cameraBackgroundRangeLabel}
        minimumMs={CAMERA_BACKGROUND_RANGE_MINIMUM_MS}
        onActivate={() => {
          onSelectTrack();
          lane.onActivate?.();
        }}
        onChange={lane.onChange}
        onClearSelection={lane.onClearSelection}
        onDraftChange={lane.onDraftChange}
        onSeek={onSeek}
        onSelect={lane.onSelect}
        onSelectSwept={lane.onSelectSwept}
        pendingSpans={lane.pendingSpans}
        selectedIds={lane.selectedIds}
        sourceDurationMs={sourceDurationMs}
        viewport={viewport}
      />
    </div>
  );
}
