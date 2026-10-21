// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { UnfoldHorizontal } from "lucide-react";
import { type ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { AudioMeter } from "../../../primitives/audio-meter/audio-meter";
import { Waveform } from "../shared/picture";

import {
  at,
  clock,
  CUT,
  MOMENTS,
  PLAYHEAD_S,
  span,
  SPEED,
  STRETCHES,
  TIMELINE_LABEL_WIDTH,
} from "./editor-data";

// The timeline's frame and the pieces its rows are built from. Monochrome:
// clips are told apart by where they sit and what they say, not by hue. Only
// the playhead is orange, where you are, and only a moment keeps its own
// colour, which names its kind.

/** A horizontal slice of a row, placed by its classes. */
export function Band({
  children,
  className,
}: {
  children: ReactNode;
  className: string;
}) {
  return (
    <span className={cn("absolute inset-x-0", className)}>{children}</span>
  );
}

/** A row: its name in the label column, then the time axis. */
export function Lane({
  children,
  className,
  label,
}: {
  label: ReactNode;
  children?: ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("flex items-center", className)}>
      <span
        className="flex h-full shrink-0 items-center pe-inline text-caption text-fg-secondary"
        style={{ width: TIMELINE_LABEL_WIDTH }}
      >
        {label}
      </span>
      <div className="relative h-full grow" dir="ltr">
        {children}
      </div>
    </div>
  );
}

/** A sound track either side of the cut; its waveform takes the text
 * colour given. */
export function Sound({
  className,
  seed,
}: {
  className: string;
  seed: number;
}) {
  return STRETCHES.map(([start, end], index) => (
    <span
      className={cn("absolute inset-y-px flex rounded-sm px-0.5", className)}
      key={start}
      style={span(start, end)}
    >
      <Waveform seed={seed + index * 2} />
    </span>
  ));
}

// Times every 30 s, moments as dots in their kind's colour, the speed as
// plain text, and the button that restores a cut over the place it closed.
function Ruler({ labelWidth }: { labelWidth: number }) {
  return (
    <div className="flex h-5 items-end">
      <span className="shrink-0" style={{ width: labelWidth }} />
      <div
        className="relative h-full grow text-caption text-fg-tertiary"
        dir="ltr"
      >
        {[0, 30, 60, 90].map((second) => (
          <span
            className="absolute bottom-0 border-s border-divider ps-1 leading-3 tabular-nums"
            key={second}
            style={{ insetInlineStart: at(second) }}
          >
            {clock(second)}
          </span>
        ))}
        <span
          className="absolute top-0 flex h-2.5 items-center justify-center border-x border-divider text-[9px] leading-none"
          style={span(SPEED.start, SPEED.end)}
        >
          {SPEED.rate}
        </span>
        {MOMENTS.map(({ at: second, color, name }) => (
          <span
            aria-label={`${name}, ${clock(second)}`}
            className="absolute top-0.5 size-1.5 -translate-x-1/2 rounded-full"
            key={second}
            style={{ background: color, insetInlineStart: at(second) }}
          />
        ))}
        <span
          aria-label={`Restore the ${String(CUT.seconds)} s cut at ${clock(CUT.at)}`}
          className="absolute bottom-0 flex size-4 -translate-x-1/2 items-center justify-center rounded-sm text-fg-secondary control-raised"
          style={{ insetInlineStart: at(CUT.at) }}
        >
          <UnfoldHorizontal className="size-3" />
        </span>
      </div>
    </div>
  );
}

// The playhead over every row under the ruler: where you are, the one place
// the orange stands for in the timeline.
function Playhead({ labelWidth }: { labelWidth: number }) {
  return (
    <div
      className="pointer-events-none absolute inset-y-0 end-0"
      dir="ltr"
      style={{ insetInlineStart: labelWidth }}
    >
      <span
        className="absolute -top-0.5 bottom-0 w-0.5 -translate-x-1/2 bg-emphasis"
        style={{ insetInlineStart: at(PLAYHEAD_S) }}
      >
        <span className="absolute -top-0.5 start-1/2 size-2.5 -translate-x-1/2 rounded-full bg-emphasis" />
      </span>
    </div>
  );
}

/** Rows under the ruler, the playhead over them and the level beside. A
 * timeline without row names passes a `labelWidth` of 0. */
export function TimelineFrame({
  children,
  labelWidth = TIMELINE_LABEL_WIDTH,
}: {
  children: ReactNode;
  labelWidth?: number;
}) {
  return (
    <div className="flex gap-inline">
      <div className="relative flex grow flex-col gap-1">
        <Ruler labelWidth={labelWidth} />
        {children}
        <Playhead labelWidth={labelWidth} />
      </div>
      <span className="mt-5 flex [&_svg]:block">
        <AudioMeter
          aria-label="Output level"
          decibels={-14}
          hidePeakTick
          hideTicks
          length="100%"
          orientation="vertical"
          thickness={4}
        />
      </span>
    </div>
  );
}
