// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Magnet,
  Maximize2,
  Scissors,
  SquareDashed,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import { RefObject } from "react";

import { IconButton } from "../../../components/base/button/icon-button";
import { NativeTooltipTrigger } from "../../../components/shared/native-tooltip/native-tooltip-trigger";
import { ToolToggle } from "../../../components/shared/tool-toggle/tool-toggle";
import { t } from "../../../i18n/i18n";

import { TimelineBladeController } from "./editing/timeline-blade";
import { VisibleRecordingMoment } from "./moments/recording-moments";
import { TimelineMomentPins } from "./moments/timeline-moment-pins";
import { Playhead } from "./scrub-playhead";
import { TimelineRuler } from "./timeline-ruler";
import { SeekHandler } from "./timeline-seek";
import { TIMELINE_MAX_ZOOM, TimelineViewportState } from "./timeline-viewport";

export function TimelineZoomToolbar({
  isBladeActive,
  isRangeActive,
  isSnapActive,
  onBladeActiveChange,
  onFit,
  onRangeActiveChange,
  onSnapActiveChange,
  onZoom,
  viewport,
}: {
  isBladeActive: boolean;
  isRangeActive: boolean;
  isSnapActive: boolean;
  onBladeActiveChange: (active: boolean) => void;
  onFit: () => void;
  onRangeActiveChange: (active: boolean) => void;
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
        isSnapActive={blade.isSnapActive}
        onBladeActiveChange={blade.setActive}
        onFit={onFit}
        onRangeActiveChange={blade.setRangeActive}
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
        <TimelineMomentPins
          artifactId={blade.edit.artifactId}
          durationMs={durationMs}
          moments={moments}
          onSeek={onSeek}
          viewport={viewport}
        />
      </div>
    </div>
  );
}
