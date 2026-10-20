// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Check,
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
import { NativeTooltipTrigger } from "../../../composites/native-tooltip/native-tooltip-trigger";
import { ToolToggle } from "../../../composites/tool-toggle/tool-toggle";
import { t } from "../../../i18n/i18n";
import { IconButton } from "../../../primitives/button/icon-button";

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
      <ToolToggle
        isSelected={isBladeActive}
        label={t("editor-timeline-blade")}
        name={t("editor-timeline-blade-name")}
        onSelectedChange={onBladeActiveChange}
        shortcut="B"
      >
        <Scissors />
      </ToolToggle>
      <ToolToggle
        isSelected={isRangeActive}
        label={t("editor-timeline-range")}
        name={t("editor-timeline-range-name")}
        onSelectedChange={onRangeActiveChange}
        shortcut="Shift+R"
      >
        <SquareDashed />
      </ToolToggle>
      <ToolToggle
        isSelected={isSnapActive}
        label={t("editor-timeline-snap")}
        name={t("editor-timeline-snap-name")}
        onSelectedChange={onSnapActiveChange}
        shortcut="Shift+S"
      >
        <Magnet />
      </ToolToggle>
      {/* The arrow keeps pointing right in every language: the timeline runs
          left to right in all of them. */}
      <NativeTooltipTrigger tooltip={t("editor-timeline-restore-order")}>
        <ConfirmActionButton
          armedIcon={<Check />}
          armedLabel={t("editor-timeline-restore-order-confirm")}
          idleIcon={<LayoutArrowRight />}
          idleLabel={t("editor-timeline-restore-order")}
          isDisabled={!isReordered}
          onConfirm={onRestoreOrder}
        />
      </NativeTooltipTrigger>
      <div className="ml-auto flex items-center gap-control">
        <NativeTooltipTrigger tooltip={t("editor-timeline-zoom-out")}>
          <IconButton
            aria-label={t("editor-timeline-zoom-out-label")}
            isDisabled={viewport.zoom <= 1}
            onPress={() => {
              onZoom(0.8);
            }}
          >
            <ZoomOut />
          </IconButton>
        </NativeTooltipTrigger>
        <NativeTooltipTrigger
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
        </NativeTooltipTrigger>
        <NativeTooltipTrigger tooltip={t("editor-timeline-zoom-in")}>
          <IconButton
            aria-label={t("editor-timeline-zoom-in-label")}
            isDisabled={viewport.zoom >= TIMELINE_MAX_ZOOM}
            onPress={() => {
              onZoom(1.25);
            }}
          >
            <ZoomIn />
          </IconButton>
        </NativeTooltipTrigger>
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
