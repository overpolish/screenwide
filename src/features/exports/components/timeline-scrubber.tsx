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
    </>
  );
}
