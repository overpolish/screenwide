// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PointerEvent as ReactPointerEvent, useRef } from "react";

import { RecordingTimelineThumbnail, RecordingVideoTrackId } from "../types";

import { TimelineBladeController, TimelineSegments } from "./timeline-blade";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import { VideoThumbnailStrip } from "./video-thumbnail-strip";

export function TimelineVideoClip({
  blade,
  enabled,
  onSelect,
  selected,
  thumbnails,
  trackId,
  viewport,
}: {
  blade: TimelineBladeController;
  enabled: boolean;
  onSelect: (trackId: RecordingVideoTrackId) => void;
  selected: boolean;
  thumbnails: RecordingTimelineThumbnail[];
  trackId: RecordingVideoTrackId;
  viewport: TimelineViewportState;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  // Unclamped: a trim drag that runs past the lane edge needs the overshoot,
  // and the blade's own conversions clamp what they are given.
  const outputPositionAt = (clientX: number) => {
    const bounds = rootRef.current?.getBoundingClientRect();
    return bounds ? timelineXToFraction(clientX, viewport, bounds) : 0;
  };

  return (
    <div
      aria-selected={selected}
      // No fill of its own: where no segment stands the lane is empty, so a
      // cut leaves a gap rather than a band of backing colour.
      className="relative h-control-height min-w-0 grow cursor-default overflow-hidden rounded-control"
      onClick={() => {
        // Only a press that no segment took gets here, so this is the lane
        // itself: the track stays chosen and the segment choice is dropped.
        if (!blade.isActive) blade.selectSegment(null);
      }}
      // Taken on the way down: a segment stops the click it handles, and a
      // trim handle suppresses the click entirely, so the press is the only
      // point at which every part of the lane can choose the track.
      onPointerDownCapture={(event: ReactPointerEvent<HTMLDivElement>) => {
        if (event.button === 0 && !blade.isActive) onSelect(trackId);
      }}
      ref={rootRef}
    >
      <TimelineViewportContent viewport={viewport}>
        <TimelineSegments
          blade={blade}
          edit={blade.edit}
          isBladeActive={blade.isActive}
          onSelectSegment={(segmentId) => {
            blade.selectSegment(segmentId);
            onSelect(trackId);
          }}
          outputPositionAt={outputPositionAt}
          renderContent={() => (
            <VideoThumbnailStrip enabled={enabled} thumbnails={thumbnails} />
          )}
          selectedSegmentId={blade.selectedSegmentId}
        />
      </TimelineViewportContent>
    </div>
  );
}
