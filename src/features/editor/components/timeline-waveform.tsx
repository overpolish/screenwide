// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PointerEvent as ReactPointerEvent, useMemo, useRef } from "react";

import { PreparedAudioTrack } from "../types";

import { TimelineBladeController, TimelineSegments } from "./timeline-blade";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import { timelineWaveformPath } from "./timeline-waveform-path";

export function Waveform({
  blade,
  enabled,
  onSelect,
  track,
  viewport,
  volumeDecibels,
}: {
  blade: TimelineBladeController;
  enabled: boolean;
  onSelect: () => void;
  track: PreparedAudioTrack;
  viewport: TimelineViewportState;
  volumeDecibels: number;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const path = useMemo(
    () => timelineWaveformPath(track.waveform, volumeDecibels),
    [track.waveform, volumeDecibels],
  );
  const outputPositionAt = (clientX: number) => {
    const bounds = rootRef.current?.getBoundingClientRect();
    return bounds ? timelineXToFraction(clientX, viewport, bounds) : 0;
  };

  return (
    <div
      // The segments carry the lane's fill, so a cut leaves the gap empty.
      className="relative h-control-height min-w-0 grow cursor-default overflow-hidden rounded-control"
      data-audio-stream-index={track.streamIndex}
      onClick={() => {
        if (!blade.isActive) blade.selectSegment(null);
      }}
      // The press, not the click: a segment stops its own click and a trim
      // handle suppresses one, so only this reaches the whole lane.
      onPointerDownCapture={(event: ReactPointerEvent<HTMLDivElement>) => {
        if (event.button === 0 && !blade.isActive) onSelect();
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
            onSelect();
          }}
          outputPositionAt={outputPositionAt}
          renderContent={() => (
            <svg
              aria-hidden="true"
              className={
                enabled
                  ? "size-full text-primary"
                  : "size-full text-content-fg-quaternary"
              }
              preserveAspectRatio="none"
              viewBox="0 0 1000 40"
            >
              <path
                className="stroke-current"
                d={path}
                fill="none"
                strokeWidth="2"
                vectorEffect="non-scaling-stroke"
              />
            </svg>
          )}
          selectedSegmentId={blade.selectedSegmentId}
        />
      </TimelineViewportContent>
    </div>
  );
}
