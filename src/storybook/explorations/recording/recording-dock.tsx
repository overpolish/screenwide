// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play, Square, Trash2 } from "lucide-react";

import { cn } from "../../../lib/styling";
import { AudioMeter } from "../../../primitives/audio-meter/audio-meter";
import { IconButton } from "../../../primitives/button/icon-button";
import { TargetStill } from "../shared/desktop";

// The dock's content only: the window draws its own material and corners.
// What is being recorded, at a glance: the camera as it is recorded, both
// input levels, the time and size so far. Then pause, stop and discard; stop
// is the one thing the dock is for, so it alone is primary. While paused the
// red dot, the error colour as a glyph, goes grey and the time steps back.

export function RecordingDock({ isPaused = false }: { isPaused?: boolean }) {
  return (
    <div className="flex h-10 w-max items-center gap-inline px-inset-sm text-fg">
      <TargetStill
        className="aspect-video h-7 overflow-hidden rounded-part"
        target="camera"
      />
      <span className="flex gap-0.5 [&_svg]:block" dir="ltr">
        <AudioMeter
          aria-label="Microphone level"
          decibels={isPaused ? -90 : -14}
          hidePeakTick
          hideTicks
          length={20}
          orientation="vertical"
          thickness={3}
        />
        <AudioMeter
          aria-label="System audio level"
          decibels={isPaused ? -90 : -26}
          hidePeakTick
          hideTicks
          length={20}
          orientation="vertical"
          thickness={3}
        />
      </span>
      <span className="flex items-center gap-inline px-1">
        <span
          className={cn(
            "size-2 rounded-full",
            isPaused ? "bg-fg-tertiary" : "bg-error",
          )}
        />
        <span
          className={cn(
            "text-label tabular-nums",
            isPaused && "text-fg-secondary",
          )}
        >
          00:42
        </span>
        <span className="text-caption text-fg-tertiary tabular-nums">
          48 MB
        </span>
      </span>
      <IconButton aria-label={isPaused ? "Resume" : "Pause"} variant="ghost">
        {isPaused ? <Play className="fill-current" /> : <Pause />}
      </IconButton>
      <IconButton aria-label="Stop" color="primary">
        <Square className="size-3! fill-current" />
      </IconButton>
      <IconButton aria-label="Discard" variant="ghost">
        <Trash2 />
      </IconButton>
    </div>
  );
}
