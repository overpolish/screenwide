// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { cn } from "../../../../lib/styling";
import { type Span } from "../timeline-scale";

// A beat's ends. Each end is a narrow zone just inside the beat: over it the
// pointer turns to resize and that end's handle shows, and pressing there
// chooses the beat and takes hold of the end at once, as an annotation bar
// is chosen and taken hold of in one press. Chosen, both handles show.
// Dragged in, an end trims the beat as it goes: what it passes is cut, with
// a seam to put it back, and the strip closes up behind it. Dragged out, it
// brings back what an earlier trim cut, as far as the beat was recorded and
// no further, since the time beside it is the lull's. How far it trims
// follows how far the pointer has gone, so the start, which stays where the
// beat begins as the beat shortens behind it, trims as surely as the end.
// The zones lie inside the beat, so a narrow lull beside it is still there
// to press; a beat too narrow to spare them keeps its whole width for
// choosing and moving until it is chosen.

/** The shortest a beat is trimmed to, in seconds. */
const LEAST_S = 0.2;
/** How far in from each end, in px, a press takes hold of that end. */
const ZONE_PX = 6;
/** The narrowest a beat has its end zones at, in px, unless chosen. */
const ZONED_PX = 24;

export type BeatTrim = {
  /** The part of the beat that plays now, inside its recorded span. */
  kept: Span;
  /** Told an end taken hold of, so the beat is chosen as it is trimmed. */
  onChoose: () => void;
  /** Told an end trimmed to, or brought back to, a recorded moment, as it
   * is dragged. */
  onTrim: (edge: "end" | "start", at: number) => void;
  /** How wide a second of the beat is drawn, in px: beats play as they
   * were recorded. */
  pxPerSecond: number;
  /** Where a dragged end settles, such as between two words in reach. */
  snap?: (seconds: number) => number;
};

type Edge = "end" | "start";

export function BeatTrimHandles({
  isSelected,
  kept,
  onChoose,
  onTrim,
  pad,
  px,
  pxPerSecond,
  snap = (seconds) => seconds,
  span,
}: BeatTrim & {
  isSelected: boolean;
  /** How far the beat's block sits inside its button, in px. */
  pad: number;
  /** How wide the beat is drawn, in px. */
  px: number;
  /** The beat as it was recorded. */
  span: Span;
}) {
  // The end held: where the pointer was pressed, the end then, and the
  // other end, which it never passes.
  const heldRef = useRef<{
    at: number;
    edge: Edge;
    from: number;
    other: number;
    x: number;
  } | null>(null);
  if (!isSelected && px < ZONED_PX) return null;

  return (["start", "end"] as const).map((edge) => (
    <span
      aria-hidden
      className={cn(
        "group/edge absolute inset-y-0 z-20 cursor-ew-resize touch-none",
        edge === "start" ? "start-0" : "end-0",
      )}
      key={edge}
      // The click that ends a trim does not press the beat as well.
      onClick={(event) => {
        event.stopPropagation();
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
        event.currentTarget.setPointerCapture(event.pointerId);
        if (!isSelected) onChoose();
        const at = edge === "start" ? kept.start : kept.end;
        heldRef.current = {
          at,
          edge,
          from: at,
          other: edge === "start" ? kept.end : kept.start,
          x: event.clientX,
        };
      }}
      onPointerMove={(event) => {
        const held = heldRef.current;
        if (held?.edge !== edge) return;
        const moved = snap(held.from + (event.clientX - held.x) / pxPerSecond);
        const at =
          edge === "start"
            ? Math.min(Math.max(moved, span.start), held.other - LEAST_S)
            : Math.max(Math.min(moved, span.end), held.other + LEAST_S);
        if (at === held.at) return;
        held.at = at;
        onTrim(edge, at);
      }}
      onPointerUp={() => {
        heldRef.current = null;
      }}
      // Over the seam a trim leaves at this end, so the end can still be
      // taken hold of to bring it back.
      style={{
        [edge === "start" ? "insetInlineStart" : "insetInlineEnd"]: pad,
        width: ZONE_PX,
      }}
    >
      <span
        className={cn(
          "pointer-events-none absolute top-1/2 h-5 w-1.5 -translate-y-1/2 rounded-full bg-on-primary ring-2 ring-primary transition-opacity",
          edge === "start"
            ? "start-0 -translate-x-1/2"
            : "end-0 translate-x-1/2",
          !isSelected && "opacity-0 group-hover/edge:opacity-100",
        )}
      />
    </span>
  ));
}
