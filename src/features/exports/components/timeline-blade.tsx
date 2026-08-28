// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcwClock } from "lucide-react";
import { ReactNode, useState } from "react";

import { Badge } from "../../../components/base/badge/badge";
import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
  RecordingTimelineTrimEdge,
} from "../recording-timeline-edit";

import {
  TimelineSegmentSpeedContextMenu,
  TimelineSpeedMenuState,
} from "./timeline-segment-speed-context-menu";
import { useTimelineNativeTrim } from "./use-timeline-native-trim";

export type TimelineBladeController = {
  beginTrim: (
    segmentId: number,
    edge: RecordingTimelineTrimEdge,
    outputPosition: number,
  ) => void;
  clearPreview: () => void;
  clearRangeSelection: () => void;
  cutAt: (sourcePosition: number) => void;
  edit: RecordingTimelineEdit;
  endTrim: (outputPosition: number) => void;
  isActive: boolean;
  isRangeActive: boolean;
  previewAt: (sourcePosition: number) => void;
  previewPosition: number | null;
  rangeSelection: TimelineRangeSelection | null;
  selectSegment: (segmentId: number | null) => void;
  selectedSegmentId: number | null;
  setActive: (active: boolean) => void;
  setRangeActive: (active: boolean) => void;
  setRangePlaybackRate: (playbackRate: number) => void;
  setRangeSelection: (anchor: number, focus: number) => void;
  setSegmentPlaybackRate: (segmentId: number, playbackRate: number) => void;
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
const cursorSvg = (paths: string, density: number) => {
  const iconColor = isMacOS ? "#111827" : "#f9fafb";
  const outlineColor = isMacOS ? "#f9fafb" : "#111827";
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

export const TIMELINE_BLADE_CURSOR = `${cursorImageSet(SCISSORS_PATHS)} 12 12, crosshair`;

const arrowLeftToLinePaths =
  '<path d="M3 19V5"/><path d="m13 6-6 6 6 6"/><path d="M7 12h14"/>';
const arrowRightToLinePaths =
  '<path d="M17 12H3"/><path d="m11 18 6-6-6-6"/><path d="M21 5v14"/>';
export const TRIM_LEFT_CURSOR = `${cursorImageSet(arrowLeftToLinePaths)} 12 12, ew-resize`;
export const TRIM_RIGHT_CURSOR = `${cursorImageSet(arrowRightToLinePaths)} 12 12, ew-resize`;

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
  const [speedMenu, setSpeedMenu] = useState<
    (TimelineSpeedMenuState & { segmentId: number }) | null
  >(null);
  const handleEvents = isBladeActive
    ? "pointer-events-none"
    : "pointer-events-auto transition hover:bg-info/40 active:bg-info/55";
  const beginTrim = useTimelineNativeTrim({ blade, outputPositionAt });
  const layout = layoutRecordingTimelineSegments(edit);
  const segments = layout.map((segment, index) => {
    const duration = segment.sourceEnd - segment.sourceStart;
    const isSelected = segment.id === selectedSegmentId;
    return (
      <div
        aria-label={`Timeline segment ${(index + 1).toString()}`}
        aria-pressed={isSelected}
        className={`absolute inset-y-0 overflow-hidden rounded-sm bg-muted/8 ${isBladeActive ? "pointer-events-none" : "pointer-events-auto"}`}
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
          const timelineTop =
            event.currentTarget
              .closest("section[aria-label='Recording timeline']")
              ?.getBoundingClientRect().top ?? 0;
          setSpeedMenu({
            segmentId: segment.id,
            x: Math.min(event.clientX, window.innerWidth - 120),
            y: Math.max(
              timelineTop + 4,
              Math.min(event.clientY, window.innerHeight - 220),
            ),
          });
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
        {(segment.playbackRate ?? 1) !== 1 ? (
          <span
            className="pointer-events-none absolute top-1 left-1/2 z-20 -translate-x-1/2"
            title={`Segment speed: ${(segment.playbackRate ?? 1).toString()}×`}
          >
            <Badge>
              <RotateCcwClock size={10} />
              {(segment.playbackRate ?? 1).toString()}×
            </Badge>
          </span>
        ) : null}
        <span
          className={`absolute inset-y-0 left-0 z-10 w-2.5 bg-info/25 ${handleEvents}`}
          onPointerDown={(event) => {
            beginTrim(segment.id, "start", event);
          }}
          style={{ cursor: TRIM_LEFT_CURSOR }}
        />
        <span
          className={`absolute inset-y-0 right-0 z-10 w-2.5 bg-info/25 ${handleEvents}`}
          onPointerDown={(event) => {
            beginTrim(segment.id, "end", event);
          }}
          style={{ cursor: TRIM_RIGHT_CURSOR }}
        />
      </div>
    );
  });
  const menuSegment = speedMenu
    ? layout.find((segment) => segment.id === speedMenu.segmentId)
    : null;
  return (
    <>
      {segments}
      {speedMenu && menuSegment ? (
        <TimelineSegmentSpeedContextMenu
          menu={speedMenu}
          onChange={(playbackRate) => {
            blade.setSegmentPlaybackRate(speedMenu.segmentId, playbackRate);
          }}
          onClose={() => {
            setSpeedMenu(null);
          }}
          playbackRate={menuSegment.playbackRate ?? 1}
          title="Segment"
        />
      ) : null}
    </>
  );
}

export function TimelineBladePreview({
  blade,
}: {
  blade: TimelineBladeController;
}) {
  return blade.isActive && blade.previewPosition !== null ? (
    <span
      aria-hidden
      className="pointer-events-none absolute inset-y-0 z-20 w-px -translate-x-1/2 bg-content-fg/40"
      style={{ left: `${(blade.previewPosition * 100).toString()}%` }}
    />
  ) : null;
}
