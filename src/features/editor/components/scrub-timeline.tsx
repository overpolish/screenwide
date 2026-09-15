// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
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
import { TimelineBladeController, TimelineSegments } from "./timeline-blade";
import { ScrubPhase, SeekHandler } from "./timeline-seek";
import { speedMarkedSegments } from "./timeline-speed-markers";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import { timelineWaveformPath } from "./timeline-waveform-path";

export type { ScrubPhase, SeekHandler } from "./timeline-seek";

const TICK_INTERVALS = [1, 2, 5, 10, 15, 30, 60, 120, 300, 600];
const MINIMUM_TICK_SPACING = 70;
/**
 * Estimated width of one footnote glyph in CSS pixels. A ruler label is
 * digits and colons only, drawn on the system font's tabular advance: at the
 * 10px footnote size that is about 6px each. Estimated rather than measured
 * because it only decides whether a label still fits before the right edge.
 */
const TICK_LABEL_GLYPH_WIDTH_PX = 6;
/** Clearance kept after a label so the next tick does not touch it. */
const TICK_LABEL_CLEARANCE_PX = 4;

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

export function TimelineRuler({
  durationMs,
  edit,
  onSeek,
  playhead,
  snapPosition = (position) => position,
  viewport,
}: {
  durationMs: number;
  onSeek: SeekHandler;
  playhead: Playhead;
  viewport: TimelineViewportState;
  /** The cut timeline, for the retime strips: a clip that plays at another
   * rate is marked along the ruler, since a rate is the timeline's and not
   * any one lane's. */
  edit?: RecordingTimelineEdit;
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
  // A retime strip sits on the ruler's last line: the rate, then a line to
  // the clip's end, closed by a short upright.
  const retimeStrips = useMemo(
    () =>
      edit
        ? speedMarkedSegments(edit).map((segment) => {
            const label = `${(segment.playbackRate ?? 1).toString()}×`;
            const left =
              (segment.outputStart - viewport.panOffset) *
              viewport.zoom *
              width;
            const right =
              (segment.outputEnd - viewport.panOffset) * viewport.zoom * width;
            return {
              id: segment.id,
              label,
              left,
              width: Math.max(0, right - left),
            };
          })
        : [],
    [edit, viewport.panOffset, viewport.zoom, width],
  );

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
      className="relative h-control-height min-w-0 grow cursor-ew-resize touch-none overflow-hidden outline-none"
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
      {ticks.map((seconds) => {
        const label = formatDuration(seconds * 1_000);
        const fraction = snapPosition(seconds / Math.max(1, durationSeconds));
        const x = (fraction - viewport.panOffset) * viewport.zoom * width;
        // The ticks keep the ruler's centre line until a retime strip needs
        // the lower half, when they move up to make room for it.
        // Match the native timeline: labels always sit after their tick and
        // disappear when they would not fit, rather than flipping to the
        // other side at the trailing edge.
        const showLabel =
          x >= 0 &&
          width - x >=
            label.length * TICK_LABEL_GLYPH_WIDTH_PX + TICK_LABEL_CLEARANCE_PX;
        return (
          <div
            className={`pointer-events-none absolute flex gap-control ${
              retimeStrips.length > 0
                ? "top-0 items-start"
                : "inset-y-0 items-center"
            }`}
            key={seconds}
            style={{ left: `${x.toString()}px` }}
          >
            {/* The tick is a filled bar rather than a rule: the band carries
                no dividers. */}
            <span className="h-control-inset w-px shrink-0 bg-content-fg-quaternary" />
            {showLabel ? (
              <span className="text-footnote leading-none whitespace-nowrap text-content-fg-secondary tabular-nums">
                {label}
              </span>
            ) : null}
          </div>
        );
      })}
      {retimeStrips.map((strip) => (
        <div
          className="pointer-events-none absolute inset-y-0 overflow-hidden"
          key={strip.id}
          style={{
            left: `${strip.left.toString()}px`,
            width: `${strip.width.toString()}px`,
          }}
        >
          {/* The rate leads, the line runs from it to the clip's end, and a
              short upright at the end says where the rate stops. The line
              is laid out after the label rather than drawn under it, so no
              mask is needed to keep it out of the text. */}
          <span className="absolute inset-x-0 bottom-0 flex items-center gap-control">
            <span className="shrink-0 pl-control text-footnote leading-none whitespace-nowrap text-content-fg-secondary tabular-nums">
              {strip.label}
            </span>
            <span className="flex min-w-0 grow items-center">
              <span className="h-tight min-w-0 grow bg-content-fg-quaternary" />
              <span className="h-control-inset w-tight shrink-0 bg-content-fg-quaternary" />
            </span>
          </span>
        </div>
      ))}
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
