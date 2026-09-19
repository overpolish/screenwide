// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MouseEvent as ReactMouseEvent, ReactNode } from "react";

import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
  RecordingTimelineTrimEdge,
} from "../recording-timeline-edit";

import { TIMELINE_LANE_LEFT_CLASS } from "./recording-track-lanes-contract";
import { clamp } from "./scrub-playhead";
import {
  timelineXToFraction,
  TimelineViewportState,
} from "./timeline-viewport";
import { useTimelineNativeTrim } from "./use-timeline-native-trim";
import { useTimelineSpeedMenu } from "./use-timeline-speed-menu";

import type { TimelineSnapGesture } from "./timeline-snap";

export type TimelineBladeController = {
  beginTrim: (gesture: {
    edge: RecordingTimelineTrimEdge;
    outputPosition: number;
    segmentId: number;
    snap?: TimelineSnapGesture;
  }) => void;
  clearPreview: () => void;
  clearRangeSelection: () => void;
  cutAt: (outputPosition: number) => void;
  edit: RecordingTimelineEdit;
  endTrim: (outputPosition: number) => void;
  isActive: boolean;
  isRangeActive: boolean;
  isSnapActive: boolean;
  previewAt: (outputPosition: number) => void;
  previewPosition: number | null;
  rangeSelection: TimelineRangeSelection | null;
  selectSegment: (segmentId: number | null) => void;
  selectedSegmentId: number | null;
  setActive: (active: boolean) => void;
  setRangeActive: (active: boolean) => void;
  setRangePlaybackRate: (playbackRate: number) => void;
  setRangeSelection: (anchor: number, focus: number) => void;
  setSegmentPlaybackRate: (segmentId: number, playbackRate: number) => void;
  setSnapActive: (active: boolean) => void;
  setSnapGuidePosition: (outputPosition: number | null) => void;
  /** Where a snapped drag is held, in output time, for the guide line. */
  snapGuidePosition: number | null;
  snapPosition: (sourcePosition: number) => number;
  /** Returns the clamped output position when the drag overshot the trim. */
  updateTrim: (outputPosition: number) => number | null;
};

export type TimelineRangeSelection = {
  end: number;
  start: number;
};

// Exact Lucide geometry. Custom CSS cursors are rasterized by the WebView, so
// density variants keep OS-level cursor enlargement from starting at 24 px.
const SCISSORS_PATHS = `
  <circle cx="6" cy="6" r="3"/>
  <path d="M8.12 8.12 12 12"/>
  <path d="M20 4 8.12 15.88"/>
  <circle cx="6" cy="18" r="3"/>
  <path d="M14.8 14.8 20 20"/>
`;
const isMacOS =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
// A cursor is rasterized from a data URI, which cannot reach the document's
// custom properties, so these literals mirror the theme tokens the cursor is
// drawn from: the glyph takes a label colour (`--color-content-fg`) and its
// outline the window colour behind it (`--color-content`). macOS draws its
// cursors dark on light, Windows the other way round, so each platform takes
// the pair from the appearance its cursors read against.
const CONTENT_FG_LIGHT = "rgba(0, 0, 0, 0.85)";
const CONTENT_LIGHT = "rgb(255, 255, 255)";
const CONTENT_FG_DARK = "rgba(255, 255, 255, 0.85)";
const CONTENT_DARK = "rgb(30, 30, 30)";
const cursorSvg = (paths: string, density: number) => {
  const iconColor = isMacOS ? CONTENT_FG_LIGHT : CONTENT_FG_DARK;
  const outlineColor = isMacOS ? CONTENT_LIGHT : CONTENT_DARK;
  const size = 24 * density;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size.toString()}" height="${size.toString()}" viewBox="0 0 24 24" fill="none" stroke-linecap="round" stroke-linejoin="round"><g stroke="${outlineColor}" stroke-width="5">${paths}</g><g stroke="${iconColor}" stroke-width="2">${paths}</g></svg>`;
};
const cursorImageSet = (paths: string) =>
  `image-set(${[1, 2, 3]
    .map(
      (density) =>
        `url("data:image/svg+xml,${encodeURIComponent(cursorSvg(paths, density))}") ${density.toString()}x`,
    )
    .join(", ")})`;

const TIMELINE_BLADE_CURSOR = `${cursorImageSet(SCISSORS_PATHS)} 12 12, crosshair`;

const arrowLeftToLinePaths =
  '<path d="M3 19V5"/><path d="m13 6-6 6 6 6"/><path d="M7 12h14"/>';
const arrowRightToLinePaths =
  '<path d="M17 12H3"/><path d="m11 18 6-6-6-6"/><path d="M21 5v14"/>';
const TRIM_LEFT_CURSOR = `${cursorImageSet(arrowLeftToLinePaths)} 12 12, ew-resize`;
const TRIM_RIGHT_CURSOR = `${cursorImageSet(arrowRightToLinePaths)} 12 12, ew-resize`;

const segmentStyle = (sourceStart: number, sourceEnd: number) => ({
  left: `${(sourceStart * 100).toString()}%`,
  width: `${((sourceEnd - sourceStart) * 100).toString()}%`,
});

export function TimelineSegments({
  blade,
  edit,
  isBladeActive,
  onSelectSegment,
  outputPositionAt,
  renderContent,
  selectedSegmentId,
}: {
  blade: TimelineBladeController;
  edit: RecordingTimelineEdit;
  isBladeActive: boolean;
  onSelectSegment: (segmentId: number) => void;
  outputPositionAt: (clientX: number) => number;
  renderContent: () => ReactNode;
  selectedSegmentId: number | null;
}) {
  const openSpeedMenu = useTimelineSpeedMenu(
    "segment",
    (playbackRate, segmentId) => {
      blade.setSegmentPlaybackRate(Number(segmentId), playbackRate);
    },
  );
  const handleEvents = isBladeActive
    ? "pointer-events-none"
    : "pointer-events-auto transition hover:bg-primary/40 active:bg-primary/55";
  const beginTrim = useTimelineNativeTrim({ blade, outputPositionAt });
  const layout = layoutRecordingTimelineSegments(edit);
  const segments = layout.map((segment, index) => {
    const duration = segment.sourceEnd - segment.sourceStart;
    const isSelected = segment.id === selectedSegmentId;
    return (
      <div
        aria-label={`Timeline segment ${(index + 1).toString()}`}
        aria-pressed={isSelected}
        className={`absolute inset-y-0 overflow-hidden rounded-control bg-fill-tertiary ${isBladeActive ? "pointer-events-none" : "pointer-events-auto"}`}
        data-timeline-segment-id={segment.id}
        key={segment.id}
        onClick={(event) => {
          event.stopPropagation();
          onSelectSegment(segment.id);
        }}
        onContextMenu={(event) => {
          if (isBladeActive) return;
          event.preventDefault();
          event.stopPropagation();
          onSelectSegment(segment.id);
          void openSpeedMenu(
            { x: event.clientX, y: event.clientY },
            segment.playbackRate ?? 1,
            segment.id.toString(),
          );
        }}
        onKeyDown={(event) => {
          if (event.key !== "Enter" && event.key !== " ") return;
          event.preventDefault();
          onSelectSegment(segment.id);
        }}
        role="button"
        style={segmentStyle(segment.outputStart, segment.outputEnd)}
        tabIndex={isBladeActive ? -1 : 0}
      >
        <div
          className="pointer-events-none absolute inset-y-0"
          style={{
            left: `${((-segment.sourceStart / duration) * 100).toString()}%`,
            width: `${(100 / duration).toString()}%`,
          }}
        >
          {renderContent()}
        </div>
        {/* A picked segment is tinted inside its own bounds: the accent
            covers this segment in this lane and nothing else. */}
        {isSelected ? (
          <span
            aria-hidden
            className="pointer-events-none absolute inset-0 z-10 bg-primary/15"
          />
        ) : null}
        <span
          className={`absolute inset-y-0 left-0 z-10 w-control-inset bg-primary/25 ${handleEvents}`}
          onPointerDown={(event) => {
            beginTrim(segment.id, "start", event);
          }}
          style={{ cursor: TRIM_LEFT_CURSOR }}
        />
        <span
          className={`absolute inset-y-0 right-0 z-10 w-control-inset bg-primary/25 ${handleEvents}`}
          onPointerDown={(event) => {
            beginTrim(segment.id, "end", event);
          }}
          style={{ cursor: TRIM_RIGHT_CURSOR }}
        />
      </div>
    );
  });
  return <>{segments}</>;
}

/**
 * The blade's hit area and its preview line: one layer over the whole lanes
 * column, the way the range tool works.
 *
 * A cut acts on the timeline, not on the lane it was aimed at, so the tool
 * has no reason to live inside a lane - and a per-lane overlay leaves every
 * position between the lanes, and below the last of them, dead. This covers
 * the same rectangle the range tool does: every lane row, the gaps between
 * them, and the space under them. Positions map through the output timeline,
 * which holds only retained ranges, so a cut always lands inside a segment
 * however wide the band it was aimed at - the only position with no cut to
 * make is a join, where one already exists, and the preview hides there
 * rather than promising one.
 */
export function TimelineBladeOverlay({
  blade,
  viewport,
}: {
  blade: TimelineBladeController;
  viewport: TimelineViewportState;
}) {
  if (!blade.isActive) return null;

  const positionAt = (event: ReactMouseEvent<HTMLDivElement>) =>
    clamp(
      timelineXToFraction(
        event.clientX,
        viewport,
        event.currentTarget.getBoundingClientRect(),
      ),
      0,
      1,
    );

  return (
    <>
      <TimelineBladePreview blade={blade} viewport={viewport} />
      <div
        aria-hidden
        // Above everything the lanes draw - segments, their trim handles and
        // their badges all sit at or below z-20 - so no part of a lane can
        // take a press the blade was aimed at.
        className={`absolute right-0 bottom-0 top-control-height ${TIMELINE_LANE_LEFT_CLASS} z-30`}
        data-timeline-blade-overlay=""
        onClick={(event) => {
          const outputPosition = positionAt(event);
          blade.cutAt(outputPosition);
        }}
        onMouseLeave={blade.clearPreview}
        onMouseMove={(event) => {
          blade.previewAt(positionAt(event));
        }}
        style={{ cursor: TIMELINE_BLADE_CURSOR }}
      />
    </>
  );
}

/**
 * The cut the click would make, drawn through the full column height so it
 * reads across every lane the way the playhead does.
 */
function TimelineBladePreview({
  blade,
  viewport,
}: {
  blade: TimelineBladeController;
  viewport: TimelineViewportState;
}) {
  return blade.previewPosition !== null ? (
    <div
      aria-hidden
      className={`pointer-events-none absolute inset-y-0 right-0 ${TIMELINE_LANE_LEFT_CLASS} z-30 overflow-hidden`}
    >
      <span
        className="absolute inset-y-0 w-px -translate-x-1/2 bg-content-fg-secondary"
        style={{
          left: `${((blade.previewPosition - viewport.panOffset) * viewport.zoom * 100).toString()}%`,
        }}
      />
    </div>
  ) : null;
}
