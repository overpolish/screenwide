// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { UnfoldHorizontal } from "lucide-react";
import { type ReactNode } from "react";

import { cn } from "../../../../lib/styling";
import { Waveform } from "../shared/mock-parts";

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
} from "./combined-data";

// The timeline's frame and the pieces its rows are built from.

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
    <div className={cn("flex h-[22px] items-center", className)}>
      <span
        className="flex h-full shrink-0 items-center gap-1.5 pe-2 text-[11px] text-(--fg-2)"
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

export function Sound({
  className,
  seed,
}: {
  /** The track's fill and, through `text-*`, its waveform's colour. */
  className: string;
  seed: number;
}) {
  return STRETCHES.map(([start, end], index) => (
    <span
      className={cn("absolute inset-y-px flex rounded-[5px] px-0.5", className)}
      key={start}
      style={span(start, end)}
    >
      <Waveform seed={seed + index * 2} />
    </span>
  ));
}

/** Times every 30 s, moments as dots, the speed as plain text, and the
 * button that restores a cut over the place it closed up. */
function Ruler({ labelWidth }: { labelWidth: number }) {
  return (
    <div className="flex h-5 items-end">
      <span className="shrink-0" style={{ width: labelWidth }} />
      <div className="relative h-full grow text-[10px] text-(--fg-3)" dir="ltr">
        {[0, 30, 60, 90].map((second) => (
          <span
            className="absolute bottom-0 border-s border-(--divider) ps-1 leading-3 tabular-nums"
            key={second}
            style={{ insetInlineStart: at(second) }}
          >
            {clock(second)}
          </span>
        ))}
        <span
          className="absolute top-0 flex h-2.5 items-center justify-center border-x border-(--divider) text-[9px] font-medium text-(--fg-3)"
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
        {/* The app's restore button at a cut: the footage unfolds back. */}
        <span
          aria-label={`Restore the ${String(CUT.seconds)} s cut at ${clock(CUT.at)}`}
          className="combined-raised absolute bottom-0 flex size-4 -translate-x-1/2 items-center justify-center rounded-[4px] text-(--fg-2)"
          style={{ insetInlineStart: at(CUT.at) }}
          title={`Restore ${String(CUT.seconds)} s cut`}
        >
          <UnfoldHorizontal className="size-3" />
        </span>
      </div>
    </div>
  );
}

/** The playhead, over every row under the ruler. */
function Playhead({ labelWidth }: { labelWidth: number }) {
  return (
    <div
      className="pointer-events-none absolute inset-y-0 end-0"
      dir="ltr"
      style={{ insetInlineStart: labelWidth }}
    >
      {/* Where you are: the one place the orange stands for, beside focus
          and progress. */}
      <span
        className="absolute -top-0.5 bottom-0 w-0.5 -translate-x-1/2 bg-(--emphasis)"
        style={{ insetInlineStart: at(PLAYHEAD_S) }}
      >
        <span className="absolute -top-0.5 start-1/2 size-2.5 -translate-x-1/2 rounded-full bg-(--emphasis)" />
      </span>
    </div>
  );
}

function LevelMeter() {
  return (
    <span
      aria-label="Audio level"
      className="relative mt-5 w-1.5 shrink-0 overflow-hidden rounded-full bg-(--fill)"
    >
      <span className="absolute inset-x-0 bottom-0 h-[58%] rounded-full bg-(--level)" />
    </span>
  );
}

/** Rows under the ruler, the playhead over them and the meter beside. A
 * timeline without row names passes a `labelWidth` of 0. */
export function TimelineFrame({
  children,
  labelWidth = TIMELINE_LABEL_WIDTH,
}: {
  children: ReactNode;
  labelWidth?: number;
}) {
  return (
    <div className="flex gap-2">
      <div className="relative flex grow flex-col gap-1">
        <Ruler labelWidth={labelWidth} />
        {children}
        <Playhead labelWidth={labelWidth} />
      </div>
      <LevelMeter />
    </div>
  );
}
