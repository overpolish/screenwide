// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Maximize2,
  Scissors,
  SquareDashed,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import { ReactNode, RefObject } from "react";
import { TooltipTrigger } from "react-aria-components";

import {
  IconButton,
  IconToggleButton,
} from "../../../components/base/button/icon-button";
import { Keyboard } from "../../../components/base/keyboard/keyboard";
import { Tooltip } from "../../../components/base/tooltip/tooltip";

import { Playhead } from "./scrub-playhead";
import { SeekHandler, TimelineRuler } from "./scrub-timeline";
import { TimelineBladeController } from "./timeline-blade";
import { TimelineViewportState } from "./timeline-viewport";

function TimelineToolbarTooltip({
  children,
  isDisabled = false,
  label,
  shortcut,
}: {
  children: ReactNode;
  label: string;
  isDisabled?: boolean;
  shortcut?: string;
}) {
  return (
    <TooltipTrigger delay={400}>
      <span className="relative inline-flex">
        {children}
        {isDisabled ? <span aria-hidden className="absolute inset-0" /> : null}
      </span>
      <Tooltip placement="bottom">
        <span className="flex items-center gap-2">
          {label}
          {shortcut ? <Keyboard>{shortcut}</Keyboard> : null}
        </span>
      </Tooltip>
    </TooltipTrigger>
  );
}

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
    <div className="flex h-9 w-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-1.25rem)] shrink-0 items-center pl-1">
      <TimelineToolbarTooltip label="Blade" shortcut="B">
        <IconToggleButton
          aria-keyshortcuts="B"
          aria-label="Blade tool"
          isSelected={isBladeActive}
          onChange={onBladeActiveChange}
          size="compact"
        >
          <Scissors size={15} />
        </IconToggleButton>
      </TimelineToolbarTooltip>
      <TimelineToolbarTooltip label="Range" shortcut="Shift+R">
        <IconToggleButton
          aria-keyshortcuts="Shift+R"
          aria-label="Range tool"
          isSelected={isRangeActive}
          onChange={onRangeActiveChange}
          size="compact"
        >
          <SquareDashed size={15} />
        </IconToggleButton>
      </TimelineToolbarTooltip>
      <div className="ml-auto flex items-center gap-0.5">
        <TimelineToolbarTooltip
          isDisabled={viewport.zoom <= 1}
          label="Zoom out"
        >
          <IconButton
            aria-label="Zoom timeline out"
            isDisabled={viewport.zoom <= 1}
            onPress={() => {
              onZoom(0.8);
            }}
            size="compact"
          >
            <ZoomOut size={15} />
          </IconButton>
        </TimelineToolbarTooltip>
        <TimelineToolbarTooltip
          isDisabled={viewport.zoom === 1 && viewport.panOffset === 0}
          label="Fit timeline"
          shortcut="Shift+Z"
        >
          <IconButton
            aria-keyshortcuts="Shift+Z"
            aria-label="Fit timeline"
            isDisabled={viewport.zoom === 1 && viewport.panOffset === 0}
            onPress={onFit}
            size="compact"
          >
            <Maximize2 size={14} />
          </IconButton>
        </TimelineToolbarTooltip>
        <TimelineToolbarTooltip
          isDisabled={viewport.zoom >= 20}
          label="Zoom in"
        >
          <IconButton
            aria-label="Zoom timeline in"
            isDisabled={viewport.zoom >= 20}
            onPress={() => {
              onZoom(1.25);
            }}
            size="compact"
          >
            <ZoomIn size={15} />
          </IconButton>
        </TimelineToolbarTooltip>
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
    <div className="flex h-9 items-center gap-2">
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
          selectedSegmentId={blade.selectedSegmentId}
          snapPosition={blade.snapPosition}
          viewport={viewport}
        />
      </div>
    </div>
  );
}
