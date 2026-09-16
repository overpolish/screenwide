// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
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

import { Playhead } from "./scrub-playhead";
import { SeekHandler, TimelineRuler } from "./scrub-timeline";
import { TimelineBladeController } from "./timeline-blade";
import { TIMELINE_MAX_ZOOM, TimelineViewportState } from "./timeline-viewport";

export function TimelineZoomToolbar({
  isBladeActive,
  isRangeActive,
  onBladeActiveChange,
  onFit,
  onRangeActiveChange,
  onZoom,
  viewport,
}: {
  isBladeActive: boolean;
  isRangeActive: boolean;
  onBladeActiveChange: (active: boolean) => void;
  onFit: () => void;
  onRangeActiveChange: (active: boolean) => void;
  onZoom: (factor: number) => void;
  viewport: TimelineViewportState;
}) {
  return (
    <div className="flex h-control-height w-timeline-gutter shrink-0 items-center gap-control">
      <ToolToggle
        isSelected={isBladeActive}
        label="Blade"
        name="Blade tool"
        onSelectedChange={onBladeActiveChange}
        shortcut="B"
      >
        <Scissors />
      </ToolToggle>
      <ToolToggle
        isSelected={isRangeActive}
        label="Range"
        name="Range tool"
        onSelectedChange={onRangeActiveChange}
        shortcut="R"
      >
        <SquareDashed />
      </ToolToggle>
      <div className="ml-auto flex items-center gap-control">
        <NativeTooltipTrigger tooltip="Zoom out">
          <IconButton
            aria-label="Zoom timeline out"
            isDisabled={viewport.zoom <= 1}
            onPress={() => {
              onZoom(0.8);
            }}
          >
            <ZoomOut />
          </IconButton>
        </NativeTooltipTrigger>
        <NativeTooltipTrigger
          tooltip={{ label: "Fit timeline", shortcut: "Shift+Z" }}
        >
          <IconButton
            aria-keyshortcuts="Shift+Z"
            aria-label="Fit timeline"
            isDisabled={viewport.zoom === 1 && viewport.panOffset === 0}
            onPress={onFit}
          >
            <Maximize2 />
          </IconButton>
        </NativeTooltipTrigger>
        <NativeTooltipTrigger tooltip="Zoom in">
          <IconButton
            aria-label="Zoom timeline in"
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
  onFit,
  onSeek,
  onZoom,
  playhead,
  viewport,
}: {
  areaRef: RefObject<HTMLDivElement | null>;
  blade: TimelineBladeController;
  durationMs: number;
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
        onBladeActiveChange={blade.setActive}
        onFit={onFit}
        onRangeActiveChange={blade.setRangeActive}
        onZoom={onZoom}
        viewport={viewport}
      />
      <div className="min-w-0 grow" ref={areaRef}>
        <TimelineRuler
          durationMs={durationMs}
          edit={blade.edit}
          onSeek={onSeek}
          playhead={playhead}
          snapPosition={blade.snapPosition}
          viewport={viewport}
        />
      </div>
    </div>
  );
}
