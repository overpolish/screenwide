// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PointerEvent as ReactPointerEvent, ReactNode } from "react";
import { useFocusRing } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";

import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
  RecordingTimelineLayoutSegment,
  RecordingTimelineTrimEdge,
} from "./recording-timeline-edit";
import { TimelineBladeController } from "./timeline-blade";
import { TRIM_LEFT_CURSOR, TRIM_RIGHT_CURSOR } from "./timeline-cursors";
import { useTimelineNativeTrim } from "./use-timeline-native-trim";
import { useTimelineSpeedMenu } from "./use-timeline-speed-menu";

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
  const beginTrim = useTimelineNativeTrim({ blade, outputPositionAt });
  return (
    <>
      {layoutRecordingTimelineSegments(edit).map((segment, index) => (
        <TimelineSegment
          isBladeActive={isBladeActive}
          isSelected={segment.id === selectedSegmentId}
          key={segment.id}
          number={index + 1}
          onBeginTrim={(edge, event) => {
            beginTrim(segment.id, edge, event);
          }}
          onOpenSpeedMenu={(point) => {
            void openSpeedMenu(
              point,
              segment.playbackRate ?? 1,
              segment.id.toString(),
            );
          }}
          onSelect={() => {
            onSelectSegment(segment.id);
          }}
          renderContent={renderContent}
          segment={segment}
        />
      ))}
    </>
  );
}

/**
 * One kept stretch of the recording in a lane, with a trim handle at each
 * end. A press focuses it, so the keyboard carries on from there, but only
 * keyboard focus draws a ring: the engine's own ring would otherwise linger
 * on a pressed segment, and on a narrow one reads as a ring on its handles.
 */
function TimelineSegment({
  isBladeActive,
  isSelected,
  number,
  onBeginTrim,
  onOpenSpeedMenu,
  onSelect,
  renderContent,
  segment,
}: {
  isBladeActive: boolean;
  isSelected: boolean;
  number: number;
  onBeginTrim: (
    edge: RecordingTimelineTrimEdge,
    event: ReactPointerEvent<HTMLSpanElement>,
  ) => void;
  onOpenSpeedMenu: (point: { x: number; y: number }) => void;
  onSelect: () => void;
  renderContent: () => ReactNode;
  segment: RecordingTimelineLayoutSegment;
}) {
  const { focusProps, isFocusVisible } = useFocusRing();
  const duration = segment.sourceEnd - segment.sourceStart;
  const handleEvents = isBladeActive
    ? "pointer-events-none"
    : "pointer-events-auto transition hover:bg-primary/40 active:bg-primary/55";
  return (
    <div
      {...focusProps}
      aria-label={t("editor-timeline-segment", { number })}
      aria-pressed={isSelected}
      className={cn(
        "absolute inset-y-0 overflow-hidden rounded-control bg-fill-tertiary",
        isBladeActive ? "pointer-events-none" : "pointer-events-auto",
        focusStyles,
        elementFocusVisible,
      )}
      data-focus-visible={isFocusVisible || undefined}
      data-timeline-segment-id={segment.id}
      onClick={(event) => {
        event.stopPropagation();
        onSelect();
      }}
      onContextMenu={(event) => {
        if (isBladeActive) return;
        event.preventDefault();
        event.stopPropagation();
        onSelect();
        onOpenSpeedMenu({ x: event.clientX, y: event.clientY });
      }}
      onKeyDown={(event) => {
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        onSelect();
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
          onBeginTrim("start", event);
        }}
        style={{ cursor: TRIM_LEFT_CURSOR }}
      />
      <span
        className={`absolute inset-y-0 right-0 z-10 w-control-inset bg-primary/25 ${handleEvents}`}
        onPointerDown={(event) => {
          onBeginTrim("end", event);
        }}
        style={{ cursor: TRIM_RIGHT_CURSOR }}
      />
    </div>
  );
}
