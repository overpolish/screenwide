// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Gauge } from "lucide-react";
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
// the beat shortens behind it, trims as surely as the end. Its end dragged
// with ⌥ (Alt) retimes it instead, as a lull's end does without: every
// frame stays and the beat plays faster or slower. What the drag does is
// fixed as it is pressed.

/** The shortest a beat is trimmed to, in seconds. */
const LEAST_S = 0.2;
/** How near its recorded length, in px, a retimed end snaps to playing as
 * recorded, as a lull's does. */
const RECORDED_SNAP_PX = 8;
/** The slowest and fastest a beat plays, as the timeline allows. */
const SLOWEST = 0.25;
const FASTEST = 4;

export type BeatTrim = {
  /** Whether ⌥ is held, which turns the end into a retime. */
  isAltHeld: boolean;
  /** The part of the beat that plays now, inside its recorded span. */
  kept: Span;
  /** Told an end trimmed to, or brought back to, a recorded moment, as it
   * is dragged, with where it was when taken hold of (`from`). */
  onTrim: (edge: "end" | "start", at: number, from: number) => void;
  /** How wide a recorded second of the beat is drawn, in px. */
  pxPerSecond: number;
  /** How many times as fast as recorded the beat plays. */
  rate: number;
  /** Told an end taken hold of, before it is dragged. */
  onHold?: (edge: "end" | "start") => void;
  /** Told how many times as fast as recorded the beat plays, as its end is
   * dragged with ⌥; none where it has no footage to retime, as a gap. */
  onRetime?: (speed: number) => void;
  /** How far the ends may be drawn out, where not just to the recorded
   * span: a gap's, which grows as far as a gap may. */
  reach?: Span;
  /** Where a dragged end settles, such as between two words in reach. */
  snap?: (seconds: number) => number;
};

type Edge = "end" | "start";

/** The end held: where the pointer was pressed, the end then, and the
 * other end, which it never passes; or, retiming, the speed last told. */
type Held =
  | { at: number; edge: Edge; from: number; other: number; x: number }
  | { edge: "retime"; speed: number };

export function BeatTrimHandles({
  isAltHeld,
  kept,
  onHold,
  onRetime,
  onTrim,
  pad,
  pxPerSecond,
  rate,
  reach,
  snap = (seconds) => seconds,
  span,
}: BeatTrim & {
  /** How far the beat's block sits inside its button, in px. */
  pad: number;
  /** The beat as it was recorded. */
  span: Span;
}) {
  const heldRef = useRef<Held | null>(null);
  const recorded = kept.end - kept.start;

  return (["start", "end"] as const).map((edge) => {
    const retimes = edge === "end" && onRetime !== undefined;
    return (
      <span
        aria-hidden
        // Over the seam a trim leaves at this end and the lull beside it, so
        // the end can still be taken hold of to bring it back.
        className={cn(
          "absolute top-1/2 z-20 flex h-7 w-3 -translate-y-1/2 cursor-ew-resize touch-none items-center justify-center",
          edge === "start" ? "-translate-x-1/2" : "translate-x-1/2",
          retimes && isAltHeld && "w-5 cursor-col-resize",
        )}
        // So the scissors' range drag on the strip, which ⌥ brings, leaves a
        // press here to the beat's own retime.
        data-retime-end={retimes ? "" : undefined}
        key={edge}
        // The click that ends a trim does not press the beat as well.
        onClick={(event) => {
          event.stopPropagation();
        }}
        onPointerDown={(event) => {
          event.stopPropagation();
          event.currentTarget.setPointerCapture(event.pointerId);
          if (retimes && event.altKey) {
            heldRef.current = { edge: "retime", speed: rate };
            return;
          }
          const at = edge === "start" ? kept.start : kept.end;
          heldRef.current = {
            at,
            edge,
            from: at,
            other: edge === "start" ? kept.end : kept.start,
            x: event.clientX,
          };
          onHold?.(edge);
        }}
        onPointerMove={(event) => {
          const held = heldRef.current;
          if (held?.edge === "retime") {
            // Measured against the beat's block as drawn now, which is as
            // long as it plays, so the end stays under the pointer as the
            // beat stretches or tightens, as a lull's does.
            const box =
              event.currentTarget.parentElement?.getBoundingClientRect();
            const drawn = box ? box.width - 2 * pad : 0;
            if (!box || drawn <= 0) return;
            const plays = recorded / rate;
            const to = (plays * (event.clientX - box.left - pad)) / drawn;
            // Drawn to within a few px of its recorded length, it plays as
            // recorded again.
            const speed =
              (Math.abs(recorded - to) / plays) * drawn < RECORDED_SNAP_PX
                ? 1
                : Math.min(
                    Math.max(recorded / Math.max(to, 1e-6), SLOWEST),
                    FASTEST,
                  );
            if (speed === held.speed) return;
            held.speed = speed;
            onRetime?.(speed);
            return;
          }
          if (held?.edge !== edge) return;
          const moved = snap(
            held.from + (event.clientX - held.x) / pxPerSecond,
          );
          const bounds = reach ?? span;
          const at =
            edge === "start"
              ? Math.min(Math.max(moved, bounds.start), held.other - LEAST_S)
              : Math.max(Math.min(moved, bounds.end), held.other + LEAST_S);
          if (at === held.at) return;
          held.at = at;
          onTrim(edge, at, held.from);
        }}
        onPointerUp={() => {
          heldRef.current = null;
        }}
        style={{
          [edge === "start" ? "insetInlineStart" : "insetInlineEnd"]: pad,
        }}
      >
        {retimes && isAltHeld ? (
          // Drawn as a chosen lull's end is with ⌥ (`spring-end.tsx`), its
          // glyph saying what the drag does.
          <span className="flex size-5 items-center justify-center rounded-full bg-on-primary text-primary ring-2 ring-primary">
            <Gauge className="size-icon-xs" />
          </span>
        ) : (
          <span className="h-5 w-1.5 rounded-full bg-on-primary ring-2 ring-primary" />
        )}
      </span>
    );
  });
}
