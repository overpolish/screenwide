// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Maximize2, Scissors, ZoomIn, ZoomOut } from "lucide-react";
import { ReactNode, RefObject } from "react";
import { TooltipTrigger } from "react-aria-components";

import { Button } from "../../../components/base/button/button";
import { ToggleButton } from "../../../components/base/button/toggle-button";
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
          {shortcut ? (
            <Keyboard size="xs" variant="tooltip">
              {shortcut}
            </Keyboard>
          ) : null}
        </span>
      </Tooltip>
    </TooltipTrigger>
  );
}

export function TimelineZoomToolbar({
  isBladeActive,
  onBladeActiveChange,
  onFit,
  onZoom,
  viewport,
}: {
  isBladeActive: boolean;
  onBladeActiveChange: (active: boolean) => void;
  onFit: () => void;
  onZoom: (factor: number) => void;
  viewport: TimelineViewportState;
}) {
  return (
    <div className="flex h-9 w-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-1.25rem)] shrink-0 items-center pl-1">
      <TimelineToolbarTooltip label="Blade" shortcut="B">
        <ToggleButton
          animation="scale-selected"
          aria-keyshortcuts="B"
          aria-label="Blade tool"
          isSelected={isBladeActive}
          onChange={onBladeActiveChange}
          showFocus={false}
          size="sm"
          variant="ghost"
        >
          <Scissors size={15} />
        </ToggleButton>
      </TimelineToolbarTooltip>
      <div className="ml-auto flex items-center gap-0.5">
        <TimelineToolbarTooltip
          isDisabled={viewport.zoom <= 1}
          label="Zoom out"
        >
          <Button
            aria-label="Zoom timeline out"
            color="muted"
            icon
            isDisabled={viewport.zoom <= 1}
            onPress={() => {
              onZoom(0.8);
            }}
            size="sm"
            variant="ghost"
          >
            <ZoomOut size={15} />
          </Button>
        </TimelineToolbarTooltip>
        <TimelineToolbarTooltip
          isDisabled={viewport.zoom === 1 && viewport.panOffset === 0}
          label="Fit timeline"
          shortcut="Shift+Z"
        >
          <Button
            aria-keyshortcuts="Shift+Z"
            aria-label="Fit timeline"
            color="muted"
            icon
            isDisabled={viewport.zoom === 1 && viewport.panOffset === 0}
            onPress={onFit}
            size="sm"
            variant="ghost"
          >
            <Maximize2 size={14} />
          </Button>
        </TimelineToolbarTooltip>
        <TimelineToolbarTooltip
          isDisabled={viewport.zoom >= 20}
          label="Zoom in"
        >
          <Button
            aria-label="Zoom timeline in"
            color="muted"
            icon
            isDisabled={viewport.zoom >= 20}
            onPress={() => {
              onZoom(1.25);
            }}
            size="sm"
            variant="ghost"
          >
            <ZoomIn size={15} />
          </Button>
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
        onBladeActiveChange={blade.setActive}
        onFit={onFit}
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
