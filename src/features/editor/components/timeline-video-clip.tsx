// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MouseEvent, useRef } from "react";

import { RecordingTimelineThumbnail, RecordingVideoTrackId } from "../types";

import { clamp } from "./scrub-playhead";
import {
  TIMELINE_BLADE_CURSOR,
  TimelineBladeController,
  TimelineBladePreview,
  TimelineSegments,
} from "./timeline-blade";
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
  const outputPositionAt = (clientX: number) => {
    const bounds = rootRef.current?.getBoundingClientRect();
    return bounds ? timelineXToFraction(clientX, viewport, bounds) : 0;
  };
  const sourcePositionAt = (event: MouseEvent<HTMLDivElement>) =>
    clamp(
      timelineXToFraction(
        event.clientX,
        viewport,
        event.currentTarget.getBoundingClientRect(),
      ),
      0,
      1,
    );
  const handleClick = (event: MouseEvent<HTMLDivElement>) => {
    if (!blade.isActive) {
      blade.selectSegment(null);
      onSelect(trackId);
      return;
    }
    blade.cutAt(sourcePositionAt(event));
  };

  return (
    <div
      aria-selected={selected}
      className="relative h-8 min-w-0 grow cursor-default overflow-hidden rounded-sm"
      onClick={handleClick}
      onMouseLeave={blade.clearPreview}
      onMouseMove={(event) => {
        if (blade.isActive) blade.previewAt(sourcePositionAt(event));
      }}
      ref={rootRef}
      style={{ cursor: blade.isActive ? TIMELINE_BLADE_CURSOR : undefined }}
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
        <TimelineBladePreview blade={blade} />
      </TimelineViewportContent>
    </div>
  );
}
