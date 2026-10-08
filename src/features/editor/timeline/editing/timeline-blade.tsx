// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MouseEvent as ReactMouseEvent } from "react";

import { clamp } from "../scrub-playhead";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "../timeline-viewport";
import { TIMELINE_LANE_LEFT_CLASS } from "../tracks/recording-track-lanes-contract";

import {
  RecordingTimelineEdit,
  RecordingTimelineTrimEdge,
} from "./recording-timeline-edit";
import { TIMELINE_BLADE_CURSOR } from "./timeline-cursors";

import type { RecordingTimelineCut } from "./recording-timeline-cuts";
import type { TimelineSnapGesture } from "../timeline-snap";

export type TimelineBladeController = {
  beginTrim: (gesture: {
    edge: RecordingTimelineTrimEdge;
    outputPosition: number;
    segmentId: number;
    snap?: TimelineSnapGesture;
  }) => void;
  clearPreview: () => void;
  clearRangeSelection: () => void;
  cutAt: (outputPosition: number) => void;
  edit: RecordingTimelineEdit;
  endTrim: (outputPosition: number) => void;
  isActive: boolean;
  isRangeActive: boolean;
  isSnapActive: boolean;
  previewAt: (outputPosition: number) => void;
  previewPosition: number | null;
  rangeSelection: TimelineRangeSelection | null;
  /** Put back what `cut` left out. */
  restoreCut: (cut: RecordingTimelineCut) => void;
  selectSegment: (segmentId: number | null) => void;
  selectedSegmentId: number | null;
  setActive: (active: boolean) => void;
  setRangeActive: (active: boolean) => void;
  setRangePlaybackRate: (playbackRate: number) => void;
  setRangeSelection: (anchor: number, focus: number) => void;
  setSegmentPlaybackRate: (segmentId: number, playbackRate: number) => void;
  setSnapActive: (active: boolean) => void;
  setSnapGuidePosition: (outputPosition: number | null) => void;
  /** Where a snapped drag is held, in output time, for the guide line. */
  snapGuidePosition: number | null;
  snapPosition: (sourcePosition: number) => number;
  /** Returns the clamped output position when the drag overshot the trim. */
  updateTrim: (outputPosition: number) => number | null;
};

export type TimelineRangeSelection = {
  end: number;
  start: number;
};

/**
 * The blade's hit area and its preview line: one layer over the whole lanes
 * column, the way the range tool works.
 *
 * A cut acts on the timeline, not on the lane it was aimed at, so the tool
 * has no reason to live inside a lane - and a per-lane overlay leaves every
 * position between the lanes, and below the last of them, dead. This covers
 * the same rectangle the range tool does: every lane row, the gaps between
 * them, and the space under them. Positions map through the output timeline,
 * which holds only retained ranges, so a cut always lands inside a segment
 * however wide the band it was aimed at - the only position with no cut to
 * make is a join, where one already exists, and the preview hides there
 * rather than promising one.
 */
export function TimelineBladeOverlay({
  blade,
  viewport,
}: {
  blade: TimelineBladeController;
  viewport: TimelineViewportState;
}) {
  if (!blade.isActive) return null;

  const positionAt = (event: ReactMouseEvent<HTMLDivElement>) =>
    clamp(
      timelineXToFraction(
        event.clientX,
        viewport,
        event.currentTarget.getBoundingClientRect(),
      ),
      0,
      1,
    );

  return (
    <>
      <TimelineBladePreview blade={blade} viewport={viewport} />
      <div
        aria-hidden
        // Above everything the lanes draw - segments, their trim handles and
        // their badges all sit at or below z-20 - so no part of a lane can
        // take a press the blade was aimed at.
        className={`absolute right-0 bottom-0 top-control-height ${TIMELINE_LANE_LEFT_CLASS} z-30`}
        data-timeline-blade-overlay=""
        onClick={(event) => {
          const outputPosition = positionAt(event);
          blade.cutAt(outputPosition);
        }}
        onMouseLeave={blade.clearPreview}
        onMouseMove={(event) => {
          blade.previewAt(positionAt(event));
        }}
        style={{ cursor: TIMELINE_BLADE_CURSOR }}
      />
    </>
  );
}

/**
 * The cut the click would make, drawn through the full column height so it
 * reads across every lane the way the playhead does.
 */
function TimelineBladePreview({
  blade,
  viewport,
}: {
  blade: TimelineBladeController;
  viewport: TimelineViewportState;
}) {
  return blade.previewPosition !== null ? (
    <div
      aria-hidden
      className={`pointer-events-none absolute inset-y-0 right-0 ${TIMELINE_LANE_LEFT_CLASS} z-30 overflow-hidden`}
    >
      <span
        className="absolute inset-y-0 w-px -translate-x-1/2 bg-content-fg-secondary"
        style={{
          left: `${((blade.previewPosition - viewport.panOffset) * viewport.zoom * 100).toString()}%`,
        }}
      />
    </div>
  ) : null;
}
