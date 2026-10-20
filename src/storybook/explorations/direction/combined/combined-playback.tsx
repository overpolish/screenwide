// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Brackets,
  ChevronsUpDown,
  ClipboardCopy,
  Layers,
  Magnet,
  Minus,
  MoveHorizontal,
  Play,
  Plus,
  Scissors,
} from "lucide-react";
import { type ReactNode } from "react";

import { cn } from "../../../../lib/styling";

import { IconBtn } from "./combined-controls";
import { clock, DURATION_S, PLAYHEAD_S } from "./combined-data";

// The timeline's controls, each kept beside the timeline: playback just
// above it, its tools in a column at its side, its zoom at its head, and the
// picture's zoom on the picture.

/** Play, the time, the speed and Copy Frame. */
function PlaybackControls({ hasPicture }: { hasPicture: boolean }) {
  return (
    <span className="flex items-center gap-2 text-[12px]">
      <span
        aria-label="Play"
        className="combined-primary flex size-7 items-center justify-center rounded-full"
      >
        <Play className="ms-0.5 size-3.5 fill-current" />
      </span>
      <span className="tabular-nums">
        {clock(PLAYHEAD_S)}
        <span className="text-(--fg-3)"> / {clock(DURATION_S)}</span>
      </span>
      <span className="combined-raised flex h-6 items-center gap-0.5 rounded-md ps-2 pe-1 tabular-nums">
        1×
        <ChevronsUpDown className="size-3 text-(--fg-3)" />
      </span>
      {hasPicture ? (
        <IconBtn label="Copy Frame">
          <ClipboardCopy />
        </IconBtn>
      ) : null}
    </span>
  );
}

/** Zoom out, in and to fit. */
function TimelineZoom() {
  return (
    <span className="flex items-center text-(--fg-3) [&>span]:size-[18px] [&>span]:rounded-md [&_svg]:size-3">
      <IconBtn label="Zoom timeline out">
        <Minus />
      </IconBtn>
      <IconBtn label="Zoom timeline in">
        <Plus />
      </IconBtn>
      <IconBtn label="Fit timeline">
        <MoveHorizontal />
      </IconBtn>
    </span>
  );
}

/**
 * Blade, Range, Snap and Tracks in a column at the timeline's side. The
 * Tracks menu opens beside its button, upward over the picture.
 */
export function TimelineTools({
  menu,
}: {
  /** The open Tracks menu. */
  menu?: ReactNode;
}) {
  return (
    <nav
      aria-label="Timeline tools"
      className="flex w-9 shrink-0 flex-col items-center gap-0.5 pt-0.5"
    >
      <IconBtn label="Blade">
        <Scissors />
      </IconBtn>
      <IconBtn label="Range">
        <Brackets />
      </IconBtn>
      {/* A switch rather than a tool: on, it is filled, not raised. */}
      <IconBtn className="combined-accent" label="Snap to edges">
        <Magnet />
      </IconBtn>
      <span className="relative">
        <IconBtn
          className={cn(menu && "bg-(--fill-2) text-(--fg)")}
          label="Tracks"
        >
          <Layers />
        </IconBtn>
        {menu ? (
          <span className="absolute start-full bottom-0 z-10 ms-1.5">
            {menu}
          </span>
        ) : null}
      </span>
    </nav>
  );
}

/** The bar over a timeline at the window's foot: playback, the handle the
 * timeline is resized from, and its zoom. */
export function PlaybackBar({ hasPicture }: { hasPicture: boolean }) {
  return (
    <div className="relative flex h-9 items-center">
      <PlaybackControls hasPicture={hasPicture} />
      <span className="absolute top-0 start-1/2 h-[3px] w-8 -translate-x-1/2 rounded-full bg-(--divider) rtl:translate-x-1/2" />
      <span className="ms-auto">
        <TimelineZoom />
      </span>
    </div>
  );
}

/** The picture's zoom, in the workspace's corner. */
export function PreviewZoom() {
  return (
    <span
      aria-label="Preview zoom"
      className="combined-raised flex h-6 items-center gap-0.5 rounded-md ps-2 pe-1 text-[12px] tabular-nums"
    >
      100%
      <ChevronsUpDown className="size-3 text-(--fg-3)" />
    </span>
  );
}
