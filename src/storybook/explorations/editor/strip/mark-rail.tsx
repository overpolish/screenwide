// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { cn } from "../../../../lib/styling";
import { type TimedMark } from "../model/use-editor";

import { at, reach, type StripScale } from "./strip-scale";

// Every annotation across the time it shows, over the beats. Press one to
// select it and drag at once: the bar moves whole, its ends move one side.
// Ends snap to where something happens (a beat's edge, an action, a step),
// shown by a guide while they hold, since an annotation is timed by events.
// The bar's hit area is taller and wider than the hairline drawn, so a thin
// mark is still easy to take hold of.

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
  /** The recorded moment under a pointer event, through the frozen lens. */
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

  return (
    <div className="relative h-6" dir="ltr">
      {marks.map((mark) => {
        const isSelected = mark.id === selected;
        const { pad, style } = reach(scale, mark, width);
        return (
          <div
            aria-label={`Annotation ${String(mark.id)}`}
            className="group absolute inset-y-0 cursor-grab touch-none active:cursor-grabbing"
            key={mark.id}
            onPointerDown={(event) => {
              begin(event, mark, "body");
            }}
            role="button"
            style={style}
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
              style={{ insetInlineEnd: pad, insetInlineStart: pad }}
            />
            {isSelected
              ? (["start", "end"] as const).map((part) => (
                  <span
                    className="absolute top-0 flex size-6 -translate-x-1/2 cursor-ew-resize items-center justify-center"
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
