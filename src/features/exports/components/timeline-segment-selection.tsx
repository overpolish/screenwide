// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
} from "../recording-timeline-edit";

import { TimelineViewportState } from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";

function TimelineSegmentSelection({
  edit,
  rounding,
  selectedSegmentId,
}: {
  edit: RecordingTimelineEdit;
  rounding: string;
  selectedSegmentId: number | null;
}) {
  const selected = layoutRecordingTimelineSegments(edit).find(
    (segment) => segment.id === selectedSegmentId,
  );
  return selected ? (
    <span
      aria-hidden
      className={`pointer-events-none absolute inset-y-0 bg-info/15 ${rounding}`}
      style={{
        left: `${(selected.outputStart * 100).toString()}%`,
        width: `${((selected.outputEnd - selected.outputStart) * 100).toString()}%`,
      }}
    />
  ) : null;
}

export function TimelineRulerSelection({
  edit,
  selectedSegmentId,
  viewport,
}: {
  selectedSegmentId: number | null;
  viewport: TimelineViewportState;
  edit?: RecordingTimelineEdit;
}) {
  return edit ? (
    <TimelineViewportContent viewport={viewport}>
      <TimelineSegmentSelection
        edit={edit}
        rounding="rounded-t-sm"
        selectedSegmentId={selectedSegmentId}
      />
    </TimelineViewportContent>
  ) : null;
}

export function TimelineLaneSelectionOverlay({
  edit,
  selectedSegmentId,
  viewport,
}: {
  edit: RecordingTimelineEdit;
  selectedSegmentId: number | null;
  viewport: TimelineViewportState;
}) {
  return (
    <div className="pointer-events-none absolute top-9 right-0 bottom-0 left-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-0.75rem)] z-[9] overflow-hidden">
      <TimelineViewportContent viewport={viewport}>
        <TimelineSegmentSelection
          edit={edit}
          rounding="rounded-b-sm"
          selectedSegmentId={selectedSegmentId}
        />
      </TimelineViewportContent>
    </div>
  );
}
