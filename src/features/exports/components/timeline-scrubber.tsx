// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  PointerEvent as ReactPointerEvent,
  useCallback,
  useEffect,
  useRef,
} from "react";

import { clamp, Playhead } from "./scrub-playhead";
import { ScrubPhase, SeekHandler } from "./scrub-timeline";
import { TimelineBladeController } from "./timeline-blade";
import { TimelineLaneSelectionOverlay } from "./timeline-segment-selection";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "./timeline-viewport";

export function TimelineScrubber({
  isInteractive = true,
  onSeek,
  playhead,
  viewport,
}: {
  onSeek: SeekHandler;
  playhead: Playhead;
  viewport: TimelineViewportState;
  isInteractive?: boolean;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const lineRef = useRef<HTMLDivElement>(null);
  const ratioRef = useRef(0);

  const positionLine = useCallback(
    (ratio: number) => {
      if (lineRef.current)
        lineRef.current.style.left = `${((ratio - viewport.panOffset) * viewport.zoom * 100).toString()}%`;
    },
    [viewport],
  );

  useEffect(
    () =>
      playhead.subscribe((_seconds, ratio) => {
        ratioRef.current = ratio;
        positionLine(ratio);
      }),
    [playhead, positionLine],
  );

  useEffect(() => {
    positionLine(ratioRef.current);
  }, [positionLine]);

  const seek = (
    event: ReactPointerEvent<HTMLDivElement>,
    phase: ScrubPhase,
  ) => {
    const bounds = rootRef.current?.getBoundingClientRect();
    if (!bounds) return;
    onSeek(
      clamp(timelineXToFraction(event.clientX, viewport, bounds), 0, 1),
      phase,
    );
  };

  return (
    <div
      className="pointer-events-none absolute inset-0 overflow-hidden"
      ref={rootRef}
    >
      <div
        className={`${isInteractive ? "pointer-events-auto" : "pointer-events-none"} absolute inset-y-0 w-3 -translate-x-1/2 cursor-ew-resize touch-none`}
        onPointerCancel={(event) => {
          seek(event, "end");
          if (event.currentTarget.hasPointerCapture(event.pointerId))
            event.currentTarget.releasePointerCapture(event.pointerId);
        }}
        onPointerDown={(event) => {
          if (event.button !== 0) return;
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
          event.currentTarget.releasePointerCapture(event.pointerId);
        }}
        ref={lineRef}
        style={{ left: "0%" }}
      >
        <span className="pointer-events-none absolute inset-y-0 left-1/2 w-px -translate-x-1/2 bg-content-fg/80" />
      </div>
    </div>
  );
}

export function TimelineScrubberOverlay(
  props: Parameters<typeof TimelineScrubber>[0] & {
    blade: TimelineBladeController;
  },
) {
  const { blade, ...scrubber } = props;
  return (
    <>
      <TimelineLaneSelectionOverlay
        edit={blade.edit}
        selectedSegmentId={blade.selectedSegmentId}
        viewport={props.viewport}
      />
      <div className="pointer-events-none absolute inset-y-0 right-0 left-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-0.75rem)] z-[5] overflow-hidden">
        <TimelineScrubber isInteractive={!blade.isActive} {...scrubber} />
      </div>
      <TimelineRangeOverlay blade={blade} viewport={props.viewport} />
    </>
  );
}

function TimelineRangeOverlay({
  blade,
  viewport,
}: {
  blade: TimelineBladeController;
  viewport: TimelineViewportState;
}) {
  const anchorRef = useRef<number | null>(null);
  if (!blade.isRangeActive) return null;

  const positionAt = (event: ReactPointerEvent<HTMLDivElement>) =>
    clamp(
      timelineXToFraction(
        event.clientX,
        viewport,
        event.currentTarget.getBoundingClientRect(),
      ),
      0,
      1,
    );
  const update = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (anchorRef.current === null) return;
    blade.setRangeSelection(anchorRef.current, positionAt(event));
  };

  return (
    <>
      {blade.rangeSelection ? (
        <div
          aria-hidden
          className="pointer-events-none absolute inset-y-0 right-0 left-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-0.75rem)] z-10 overflow-hidden"
        >
          <div
            className="absolute inset-y-0 border-x border-info/70 bg-info/15"
            style={{
              left: `${((blade.rangeSelection.start - viewport.panOffset) * viewport.zoom * 100).toString()}%`,
              width: `${((blade.rangeSelection.end - blade.rangeSelection.start) * viewport.zoom * 100).toString()}%`,
            }}
          />
        </div>
      ) : null}
      <div
        aria-label="Select timeline range"
        className="absolute top-9 right-0 bottom-0 left-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-0.75rem)] z-10 cursor-crosshair touch-none"
        onPointerCancel={(event) => {
          update(event);
          anchorRef.current = null;
          if (event.currentTarget.hasPointerCapture(event.pointerId))
            event.currentTarget.releasePointerCapture(event.pointerId);
        }}
        onPointerDown={(event) => {
          if (event.button !== 0) return;
          event.preventDefault();
          anchorRef.current = positionAt(event);
          blade.clearRangeSelection();
          event.currentTarget.setPointerCapture(event.pointerId);
        }}
        onPointerMove={update}
        onPointerUp={(event) => {
          update(event);
          anchorRef.current = null;
          if (event.currentTarget.hasPointerCapture(event.pointerId))
            event.currentTarget.releasePointerCapture(event.pointerId);
        }}
      />
    </>
  );
}
