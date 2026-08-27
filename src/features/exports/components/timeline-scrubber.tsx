// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  MouseEvent as ReactMouseEvent,
  PointerEvent as ReactPointerEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { recordingTimelineRangePlaybackRate } from "../recording-timeline-speed";

import { clamp, Playhead } from "./scrub-playhead";
import { TimelineBladeController } from "./timeline-blade";
import { TimelineLaneSelectionOverlay } from "./timeline-segment-selection";
import {
  TimelineSegmentSpeedContextMenu,
  TimelineSpeedMenuState,
} from "./timeline-segment-speed-context-menu";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "./timeline-viewport";

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
        className="absolute inset-y-0 w-px -translate-x-1/2 bg-content-fg/80"
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
      <TimelineLaneSelectionOverlay
        edit={blade.edit}
        selectedSegmentId={blade.selectedSegmentId}
        viewport={props.viewport}
      />
      <div className="pointer-events-none absolute inset-y-0 right-0 left-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-0.75rem)] z-[5] overflow-hidden">
        <TimelineScrubber {...scrubber} />
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
  const [speedMenu, setSpeedMenu] = useState<TimelineSpeedMenuState | null>(
    null,
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
          const timelineTop =
            event.currentTarget
              .closest("section[aria-label='Recording timeline']")
              ?.getBoundingClientRect().top ?? 0;
          setSpeedMenu({
            x: Math.min(event.clientX, window.innerWidth - 120),
            y: Math.max(
              timelineTop + 4,
              Math.min(event.clientY, window.innerHeight - 220),
            ),
          });
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
      {speedMenu && blade.rangeSelection ? (
        <TimelineSegmentSpeedContextMenu
          menu={speedMenu}
          onChange={blade.setRangePlaybackRate}
          onClose={() => {
            setSpeedMenu(null);
          }}
          playbackRate={recordingTimelineRangePlaybackRate(
            blade.edit,
            blade.rangeSelection.start,
            blade.rangeSelection.end,
          )}
          title="Range"
        />
      ) : null}
    </>
  );
}
