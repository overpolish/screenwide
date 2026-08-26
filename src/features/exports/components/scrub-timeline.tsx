// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  MouseEvent as ReactMouseEvent,
  PointerEvent as ReactPointerEvent,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { PREVIEW_FRAME_MS, formatDuration } from "../duration";
import { RecordingTimelineEdit } from "../recording-timeline-edit";
import { PreparedAudioTrack } from "../types";

import { clamp, Playhead } from "./scrub-playhead";
import {
  TIMELINE_BLADE_CURSOR,
  TimelineBladeController,
  TimelineBladePreview,
  TimelineSegments,
} from "./timeline-blade";
import { TimelineRulerSelection } from "./timeline-segment-selection";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import { timelineWaveformPath } from "./timeline-waveform-path";

export type ScrubPhase = "end" | "move" | "start";
export type SeekHandler = (ratio: number, phase: ScrubPhase) => void;

const TICK_INTERVALS = [1, 2, 5, 10, 15, 30, 60, 120, 300, 600];
const MINIMUM_TICK_SPACING = 70;

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

  return (
    <div
      className="relative h-8 min-w-0 grow cursor-default overflow-hidden rounded-sm"
      data-audio-stream-index={track.streamIndex}
      onClick={(event: ReactMouseEvent<HTMLDivElement>) => {
        if (!blade.isActive) {
          blade.selectSegment(null);
          onSelect();
          return;
        }
        const bounds = event.currentTarget.getBoundingClientRect();
        blade.cutAt(
          clamp(timelineXToFraction(event.clientX, viewport, bounds), 0, 1),
        );
      }}
      onMouseLeave={blade.clearPreview}
      onMouseMove={(event) => {
        if (!blade.isActive) return;
        const bounds = event.currentTarget.getBoundingClientRect();
        blade.previewAt(
          clamp(timelineXToFraction(event.clientX, viewport, bounds), 0, 1),
        );
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
            onSelect();
          }}
          outputPositionAt={(clientX) => {
            const bounds = rootRef.current?.getBoundingClientRect();
            return bounds ? timelineXToFraction(clientX, viewport, bounds) : 0;
          }}
          renderContent={() => (
            <svg
              aria-hidden="true"
              className={
                enabled ? "size-full text-info" : "size-full text-muted/35"
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
        <TimelineBladePreview blade={blade} />
      </TimelineViewportContent>
    </div>
  );
}

export function TimelineRuler({
  durationMs,
  edit,
  onSeek,
  playhead,
  selectedSegmentId = null,
  snapPosition = (position) => position,
  viewport,
}: {
  durationMs: number;
  onSeek: SeekHandler;
  playhead: Playhead;
  viewport: TimelineViewportState;
  edit?: RecordingTimelineEdit;
  selectedSegmentId?: number | null;
  snapPosition?: (sourcePosition: number) => number;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const ratioRef = useRef(0);
  const [width, setWidth] = useState(0);
  const durationSeconds = Math.max(0, durationMs / 1_000);
  const pixelsPerSecond =
    (width * viewport.zoom) / Math.max(1, durationSeconds);
  const interval =
    TICK_INTERVALS.find(
      (candidate) => candidate * pixelsPerSecond >= MINIMUM_TICK_SPACING,
    ) ?? TICK_INTERVALS[TICK_INTERVALS.length - 1];
  const ticks = useMemo(() => {
    if (durationSeconds <= 0) return [0];
    return Array.from(
      { length: Math.floor(durationSeconds / interval) + 1 },
      (_, index) => index * interval,
    );
  }, [durationSeconds, interval]);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const observer = new ResizeObserver(() => {
      setWidth(root.clientWidth);
    });
    observer.observe(root);
    setWidth(root.clientWidth);
    return () => {
      observer.disconnect();
    };
  }, []);

  useEffect(
    () =>
      playhead.subscribe((_seconds, ratio) => {
        ratioRef.current = ratio;
        // Assistive technology needs the position too, and this element is
        // never re-rendered, so React will not overwrite the attribute.
        rootRef.current?.setAttribute(
          "aria-valuenow",
          Math.round(ratio * 100).toString(),
        );
      }),
    [playhead],
  );

  const seek = (
    event: ReactPointerEvent<HTMLDivElement>,
    phase: ScrubPhase,
  ) => {
    const bounds = event.currentTarget.getBoundingClientRect();
    onSeek(
      clamp(timelineXToFraction(event.clientX, viewport, bounds), 0, 1),
      phase,
    );
  };

  return (
    <div
      aria-label="Recording position"
      aria-valuemax={100}
      aria-valuemin={0}
      aria-valuenow={0}
      className="relative h-9 min-w-0 grow cursor-ew-resize touch-none overflow-hidden outline-none"
      onKeyDown={(event) => {
        if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
        event.preventDefault();
        // The window shortcut handles the arrows everywhere else; matching its
        // one frame / one second steps keeps a focused ruler consistent.
        const stepMs = event.shiftKey ? 1_000 : PREVIEW_FRAME_MS;
        const ratio = clamp(
          ratioRef.current +
            (event.key === "ArrowRight" ? stepMs : -stepMs) /
              Math.max(1, durationMs),
          0,
          1,
        );
        onSeek(ratio, "start");
        onSeek(ratio, "end");
      }}
      onPointerCancel={(event) => {
        seek(event, "end");
        if (event.currentTarget.hasPointerCapture(event.pointerId))
          event.currentTarget.releasePointerCapture(event.pointerId);
        event.currentTarget.blur();
      }}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        // Pointer scrubbing must not leave a transient focus treatment on the
        // ruler; keyboard users can still reach it normally with Tab.
        event.preventDefault();
        event.currentTarget.setPointerCapture(event.pointerId);
        seek(event, "start");
      }}
      onPointerMove={(event) => {
        if (event.currentTarget.hasPointerCapture(event.pointerId))
          seek(event, "move");
      }}
      onPointerUp={(event) => {
        seek(event, "end");
        if (event.currentTarget.hasPointerCapture(event.pointerId))
          event.currentTarget.releasePointerCapture(event.pointerId);
        event.currentTarget.blur();
      }}
      ref={rootRef}
      role="slider"
      tabIndex={0}
    >
      <TimelineRulerSelection {...{ edit, selectedSegmentId, viewport }} />
      {ticks.map((seconds) => {
        const label = formatDuration(seconds * 1_000);
        const fraction = snapPosition(seconds / Math.max(1, durationSeconds));
        const x = (fraction - viewport.panOffset) * viewport.zoom * width;
        // Match the native timeline: labels always sit after their tick and
        // disappear when they would not fit, rather than flipping to the
        // other side at the trailing edge.
        const showLabel = x >= 0 && width - x >= label.length * 6 + 4;
        return (
          <div
            className="pointer-events-none absolute top-2.5 h-4 border-l border-muted/35"
            key={seconds}
            style={{ left: `${x.toString()}px` }}
          >
            {showLabel ? (
              <span className="absolute left-1 top-0 whitespace-nowrap text-xxs font-medium text-muted tabular-nums">
                {label}
              </span>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}

/** The elapsed half of the time readout, written straight to the text node. */
export function ElapsedTime({ playhead }: { playhead: Playhead }) {
  const ref = useRef<HTMLSpanElement>(null);

  useEffect(
    () =>
      playhead.subscribe((seconds) => {
        const text = formatDuration(seconds * 1000);
        if (ref.current && ref.current.textContent !== text)
          ref.current.textContent = text;
      }),
    [playhead],
  );

  return <span ref={ref}>{formatDuration(0)}</span>;
}
