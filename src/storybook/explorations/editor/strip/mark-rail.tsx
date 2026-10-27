// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Focus, type LucideIcon } from "lucide-react";
import { useRef, useState } from "react";

import {
  ArrowToolIcon,
  CounterToolIcon,
} from "../../../../features/annotations/annotation-tool-icons";
import { cn } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { type Pen } from "../../ideas/looks";
import { type TimedMark } from "../model/marks";

import { at, reach, type StripScale } from "./strip-scale";

// Every annotation across the time it shows, over the beats. Press one to
// select it and drag at once: the bar moves whole, its ends move one side.
// Ends snap to where something happens (a beat's edge, an action),
// shown by a guide while they hold, since an annotation is timed by events.
// The bar's hit area is taller and wider than the hairline drawn, so a thin
// mark is still easy to take hold of. Annotations that show at once sit in
// lanes of their own, each marked with its tool, so one is never hidden
// under another.

/** One lane's height, in px: a badge's, with a little room. */
const LANE_PX = 20;
/** The rail's height with one lane, in px, as much as a selected bar's
 * handles need. */
const MIN_RAIL_PX = 24;
/** A bar with a gap in its middle, half as wide as this many px each side,
 * where its badge sits: the badge's width with a little air. */
const GAP_UNDER_BADGE = `linear-gradient(to right, #000 calc(50% - 12px), transparent calc(50% - 12px), transparent calc(50% + 12px), #000 calc(50% + 12px))`;

const PEN_ICONS: Record<Pen, LucideIcon> = {
  arrow: ArrowToolIcon,
  counter: CounterToolIcon,
  focus: Focus,
};

/** Each annotation's lane: the first, by when it was placed, where it meets
 * none already there. Placed earlier, an annotation keeps its lane while a
 * later one is moved round it. */
function lanesOf(marks: TimedMark[]) {
  const taken: TimedMark[][] = [];
  const lanes = new Map<number, number>();
  for (const mark of [...marks].sort((first, second) => first.id - second.id)) {
    let lane = taken.findIndex((placed) =>
      placed.every(({ end, start }) => end <= mark.start || start >= mark.end),
    );
    if (lane < 0) lane = taken.push([]) - 1;
    taken[lane].push(mark);
    lanes.set(mark.id, lane);
  }
  return lanes;
}

type Drag = {
  end: number;
  grab: number;
  id: number;
  part: "body" | "end" | "start";
  start: number;
};

export type Snap = (seconds: number) => { isSnapped: boolean; seconds: number };

export function MarkRail({
  marks,
  onRetime,
  onSelect,
  scale,
  secondsAt,
  selected,
  snap,
  width,
}: {
  marks: TimedMark[];
  onRetime: (id: number, start: number, end: number) => void;
  onSelect: (id: number) => void;
  scale: StripScale;
  /** The recorded moment under a pointer event, through the lens. */
  secondsAt: (event: React.PointerEvent) => number;
  snap: Snap;
  width: number;
  selected?: number;
}) {
  const dragRef = useRef<Drag | null>(null);
  const [guide, setGuide] = useState<number | null>(null);

  const begin = (
    event: React.PointerEvent,
    mark: TimedMark,
    part: Drag["part"],
  ) => {
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    if (mark.id !== selected) onSelect(mark.id);
    dragRef.current = {
      end: mark.end,
      grab: secondsAt(event),
      id: mark.id,
      part,
      start: mark.start,
    };
  };
  const move = (event: React.PointerEvent) => {
    const drag = dragRef.current;
    if (!drag) return;
    const pointer = secondsAt(event);
    if (drag.part !== "body") {
      const edge = snap(pointer);
      setGuide(edge.isSnapped ? edge.seconds : null);
      if (drag.part === "start") onRetime(drag.id, edge.seconds, drag.end);
      else onRetime(drag.id, drag.start, edge.seconds);
      return;
    }
    const length = drag.end - drag.start;
    const moved = drag.start + pointer - drag.grab;
    const head = snap(moved);
    const tail = snap(moved + length);
    const start = head.isSnapped
      ? head.seconds
      : tail.isSnapped
        ? tail.seconds - length
        : moved;
    setGuide(
      head.isSnapped ? head.seconds : tail.isSnapped ? tail.seconds : null,
    );
    onRetime(drag.id, start, start + length);
  };
  const end = () => {
    dragRef.current = null;
    setGuide(null);
  };
  const handlers = { onPointerMove: move, onPointerUp: end };

  const lanes = lanesOf(marks);
  const laneCount = Math.max(1, ...[...lanes.values()].map((lane) => lane + 1));

  return (
    <div
      className="relative"
      dir="ltr"
      style={{ height: Math.max(MIN_RAIL_PX, laneCount * LANE_PX) }}
    >
      {marks.map((mark) => {
        const isSelected = mark.id === selected;
        const { pad, style } = reach(scale, mark, width);
        const Icon = PEN_ICONS[mark.pen];
        return (
          <div
            aria-label={`Annotation ${String(mark.id)}`}
            className="group absolute cursor-grab touch-none active:cursor-grabbing"
            key={mark.id}
            onPointerDown={(event) => {
              begin(event, mark, "body");
            }}
            role="button"
            style={{
              ...style,
              height: LANE_PX,
              top: (lanes.get(mark.id) ?? 0) * LANE_PX,
            }}
            tabIndex={-1}
            {...handlers}
          >
            <span
              className={cn(
                "absolute top-1/2 -translate-y-1/2 rounded-full transition-[height,background-color]",
                isSelected
                  ? "h-1.5 bg-primary"
                  : "h-0.5 bg-fg-tertiary group-hover:h-1 group-hover:bg-fg-secondary",
              )}
              // The bar stops short of the badge in its middle, so the
              // badge's own fill never shows the bar through it.
              style={{
                insetInlineEnd: pad,
                insetInlineStart: pad,
                maskImage: GAP_UNDER_BADGE,
              }}
            />
            {/* Which annotation it is, on a badge in the bar's own colour,
                in the middle of the bar, however short. The handles of a
                chosen bar sit at its ends, over it. */}
            <span
              aria-hidden
              className="absolute start-1/2 top-1/2 flex -translate-1/2"
            >
              <Badge
                className={cn(
                  "px-0.5 text-surface transition-colors",
                  isSelected
                    ? "bg-primary"
                    : "bg-fg-tertiary group-hover:bg-fg-secondary",
                )}
              >
                <Icon />
              </Badge>
            </span>
            {isSelected
              ? (["start", "end"] as const).map((part) => (
                  <span
                    className="absolute top-1/2 flex size-6 -translate-1/2 cursor-ew-resize items-center justify-center"
                    key={part}
                    onPointerDown={(event) => {
                      begin(event, mark, part);
                    }}
                    style={{
                      insetInlineStart:
                        part === "start"
                          ? pad
                          : `calc(100% - ${String(pad)}px)`,
                    }}
                    {...handlers}
                  >
                    <span className="size-3 rounded-full bg-primary ring-2 ring-recess" />
                  </span>
                ))
              : null}
          </div>
        );
      })}
      {guide === null ? null : (
        <span
          className="pointer-events-none absolute top-0 h-24 w-px -translate-x-1/2 border-s border-dashed border-fg-secondary"
          style={at(scale, guide)}
        />
      )}
    </div>
  );
}
