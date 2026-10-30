// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { cn } from "../../../../lib/styling";
import { type Span } from "../timeline-scale";

// A chosen beat's ends. Each handle sits on the beat's edge, and is pressed
// across its whole width: the half past the edge reaches over the lull
// beside it, as the chosen beat is what its edge belongs to. Dragged in, an
// end trims the beat as it goes: what it passes is cut, with a seam to put
// it back, and the strip closes up behind it. Dragged out, it brings back
// what an earlier trim cut, as far as the beat was recorded and no further,
// since the time beside it is the lull's. How far it trims follows how far
// the pointer has gone, so the start, which stays where the beat begins as
// the beat shortens behind it, trims as surely as the end.

/** The shortest a beat is trimmed to, in seconds. */
const LEAST_S = 0.2;

export type BeatTrim = {
  /** The part of the beat that plays now, inside its recorded span. */
  kept: Span;
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
  kept,
  onTrim,
  pad,
  pxPerSecond,
  snap = (seconds) => seconds,
  span,
}: BeatTrim & {
  /** How far the beat's block sits inside its button, in px. */
  pad: number;
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

  return (["start", "end"] as const).map((edge) => (
    <span
      aria-hidden
      // Over the seam a trim leaves at this end and the lull beside it, so
      // the end can still be taken hold of to bring it back.
      className={cn(
        "absolute top-1/2 z-20 flex h-7 w-3 -translate-y-1/2 cursor-ew-resize touch-none items-center justify-center",
        edge === "start" ? "-translate-x-1/2" : "translate-x-1/2",
      )}
      key={edge}
      // The click that ends a trim does not press the beat as well.
      onClick={(event) => {
        event.stopPropagation();
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
        event.currentTarget.setPointerCapture(event.pointerId);
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
      style={{
        [edge === "start" ? "insetInlineStart" : "insetInlineEnd"]: pad,
      }}
    >
      <span className="h-5 w-1.5 rounded-full bg-on-primary ring-2 ring-primary" />
    </span>
  ));
}
