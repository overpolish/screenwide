// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  MouseEvent as ReactMouseEvent,
  PointerEvent as ReactPointerEvent,
  useCallback,
  useEffect,
  useRef,
} from "react";

import { recordingTimelineRangePlaybackRate } from "../recording-timeline-speed";

import { TIMELINE_LANE_LEFT_CLASS } from "./recording-track-lanes-contract";
import { clamp, Playhead } from "./scrub-playhead";
import {
  TimelineBladeController,
  TimelineBladeOverlay,
} from "./timeline-blade";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "./timeline-viewport";
import { useTimelineSpeedMenu } from "./use-timeline-speed-menu";

/**
 * The playhead line across the track lanes. Purely visual: scrubbing happens
 * on the ruler, so the line never steals a click from the clip beneath it.
 */
export function TimelineScrubber({
  playhead,
  viewport,
}: {
  playhead: Playhead;
  viewport: TimelineViewportState;
}) {
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

  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      <div
        className="absolute inset-y-0 w-px -translate-x-1/2 bg-content-fg"
        ref={lineRef}
        style={{ left: "0%" }}
      />
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
      {/* Above the ruler: the playhead is the one mark that has to stay
          readable across it, and it takes no presses, so nothing below it
          loses a click to this layer. */}
      <div
        className={`pointer-events-none absolute inset-y-0 right-0 ${TIMELINE_LANE_LEFT_CLASS} z-30 overflow-hidden`}
      >
        <TimelineScrubber {...scrubber} />
      </div>
      <TimelineRangeOverlay blade={blade} viewport={props.viewport} />
      {/* One layer for the blade too, over the same rectangle: a cut acts on
          the timeline, not on the lane the pointer happened to be over. */}
      <TimelineBladeOverlay blade={blade} viewport={props.viewport} />
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
  const openSpeedMenu = useTimelineSpeedMenu(
    "range",
    blade.setRangePlaybackRate,
  );
  if (!blade.isRangeActive) return null;

  const positionAt = (
    event: ReactMouseEvent<HTMLDivElement> | ReactPointerEvent<HTMLDivElement>,
  ) =>
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
          className={`pointer-events-none absolute inset-y-0 right-0 ${TIMELINE_LANE_LEFT_CLASS} z-10 overflow-hidden`}
        >
          <div
            className="absolute inset-y-0 bg-primary/15"
            style={{
              left: `${((blade.rangeSelection.start - viewport.panOffset) * viewport.zoom * 100).toString()}%`,
              width: `${((blade.rangeSelection.end - blade.rangeSelection.start) * viewport.zoom * 100).toString()}%`,
            }}
          >
            {/* The range's own edges, drawn as filled lines rather than a
                border so they read as part of the selection, not a rule. */}
            <span className="absolute inset-y-0 left-0 w-px bg-primary" />
            <span className="absolute inset-y-0 right-0 w-px bg-primary" />
          </div>
        </div>
      ) : null}
      <div
        aria-label="Select timeline range"
        className={`absolute right-0 bottom-0 top-control-height ${TIMELINE_LANE_LEFT_CLASS} z-10 cursor-crosshair touch-none`}
        onContextMenu={(event) => {
          const selection = blade.rangeSelection;
          const position = positionAt(event);
          if (
            !selection ||
            position < selection.start ||
            position > selection.end
          )
            return;
          event.preventDefault();
          event.stopPropagation();
          void openSpeedMenu(
            { x: event.clientX, y: event.clientY },
            recordingTimelineRangePlaybackRate(
              blade.edit,
              selection.start,
              selection.end,
            ),
          );
        }}
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
