// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  LayoutArrowRight,
  Magnet,
  Maximize2,
  Scissors,
  SquareDashed,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import { RefObject } from "react";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";
import { IconButton } from "../../../primitives/button/icon-button";
import { ToolbarTool } from "../../../primitives/toolbar/toolbar-tool";
import { TooltipTrigger } from "../../../primitives/tooltip/tooltip-trigger";

import { isRecordingTimelineReordered } from "./editing/recording-timeline-move";
import { TimelineBladeController } from "./editing/timeline-blade";
import { TimelineCutMarkers } from "./editing/timeline-cut-markers";
import { VisibleRecordingMoment } from "./moments/recording-moments";
import { TimelineMomentPins } from "./moments/timeline-moment-pins";
import { Playhead } from "./scrub-playhead";
import { TimelineRuler } from "./timeline-ruler";
import { SeekHandler } from "./timeline-seek";
import { TIMELINE_MAX_ZOOM, TimelineViewportState } from "./timeline-viewport";

export function TimelineZoomToolbar({
  isBladeActive,
  isRangeActive,
  isReordered,
  isSnapActive,
  onBladeActiveChange,
  onFit,
  onRangeActiveChange,
  onRestoreOrder,
  onSnapActiveChange,
  onZoom,
  viewport,
}: {
  isBladeActive: boolean;
  isRangeActive: boolean;
  /** Whether any clip plays out of the order it was recorded in. */
  isReordered: boolean;
  isSnapActive: boolean;
  onBladeActiveChange: (active: boolean) => void;
  onFit: () => void;
  onRangeActiveChange: (active: boolean) => void;
  /** Put every clip back in the order it was recorded. */
  onRestoreOrder: () => void;
  onSnapActiveChange: (active: boolean) => void;
  onZoom: (factor: number) => void;
  viewport: TimelineViewportState;
}) {
  return (
    <div className="flex h-control-height w-timeline-gutter shrink-0 items-center gap-control">
      <ToolbarTool
        aria-label={t("editor-timeline-blade-name")}
        isSelected={isBladeActive}
        label={t("editor-timeline-blade")}
        onChange={onBladeActiveChange}
        shortcut="B"
      >
        <Scissors />
      </ToolbarTool>
      <ToolbarTool
        aria-label={t("editor-timeline-range-name")}
        isSelected={isRangeActive}
        label={t("editor-timeline-range")}
        onChange={onRangeActiveChange}
        shortcut="Shift+R"
      >
        <SquareDashed />
      </ToolbarTool>
      <ToolbarTool
        aria-label={t("editor-timeline-snap-name")}
        isSelected={isSnapActive}
        label={t("editor-timeline-snap")}
        onChange={onSnapActiveChange}
        shortcut="Shift+S"
      >
        <Magnet />
      </ToolbarTool>
      {/* The arrow keeps pointing right in every language: the timeline runs
          left to right in all of them. */}
      <TooltipTrigger tooltip={t("editor-timeline-restore-order")}>
        <ConfirmActionButton
          confirmLabel={t("editor-timeline-restore-order-confirm")}
          icon={<LayoutArrowRight />}
          isDisabled={!isReordered}
          label={t("editor-timeline-restore-order")}
          onConfirm={onRestoreOrder}
          variant="ghost"
        />
      </TooltipTrigger>
      <div className="ml-auto flex items-center gap-control">
        <TooltipTrigger tooltip={t("editor-timeline-zoom-out")}>
          <IconButton
            aria-label={t("editor-timeline-zoom-out-label")}
            isDisabled={viewport.zoom <= 1}
            onPress={() => {
              onZoom(0.8);
            }}
          >
            <ZoomOut />
          </IconButton>
        </TooltipTrigger>
        <TooltipTrigger
          tooltip={{ label: t("editor-timeline-fit"), shortcut: "Shift+KeyZ" }}
        >
          <IconButton
            aria-keyshortcuts="Shift+Z"
            aria-label={t("editor-timeline-fit")}
            isDisabled={viewport.zoom === 1 && viewport.panOffset === 0}
            onPress={onFit}
          >
            <Maximize2 />
          </IconButton>
        </TooltipTrigger>
        <TooltipTrigger tooltip={t("editor-timeline-zoom-in")}>
          <IconButton
            aria-label={t("editor-timeline-zoom-in-label")}
            isDisabled={viewport.zoom >= TIMELINE_MAX_ZOOM}
            onPress={() => {
              onZoom(1.25);
            }}
          >
            <ZoomIn />
          </IconButton>
        </TooltipTrigger>
      </div>
    </div>
  );
}

export function TimelineHeader({
  areaRef,
  blade,
  durationMs,
  moments,
  onFit,
  onSeek,
  onZoom,
  playhead,
  viewport,
}: {
  areaRef: RefObject<HTMLDivElement | null>;
  blade: TimelineBladeController;
  durationMs: number;
  /** The moments the edit keeps, pinned over the ruler. */
  moments: readonly VisibleRecordingMoment[];
  onFit: () => void;
  onSeek: SeekHandler;
  onZoom: (factor: number) => void;
  playhead: Playhead;
  viewport: TimelineViewportState;
}) {
  return (
    <div className="flex h-control-height items-center gap-section">
      <TimelineZoomToolbar
        isBladeActive={blade.isActive}
        isRangeActive={blade.isRangeActive}
        isReordered={isRecordingTimelineReordered(blade.edit)}
        isSnapActive={blade.isSnapActive}
        onBladeActiveChange={blade.setActive}
        onFit={onFit}
        onRangeActiveChange={blade.setRangeActive}
        onRestoreOrder={blade.restoreOrder}
        onSnapActiveChange={blade.setSnapActive}
        onZoom={onZoom}
        viewport={viewport}
      />
      <div className="relative min-w-0 grow" ref={areaRef}>
        <TimelineRuler
          durationMs={durationMs}
          edit={blade.edit}
          onSeek={onSeek}
          playhead={playhead}
          snapPosition={blade.snapPosition}
          viewport={viewport}
        />
        <TimelineCutMarkers
          blade={blade}
          durationMs={durationMs}
          viewport={viewport}
        />
        <TimelineMomentPins
          durationMs={durationMs}
          edit={blade.edit}
          moments={moments}
          onSeek={onSeek}
          viewport={viewport}
        />
      </div>
    </div>
  );
}
