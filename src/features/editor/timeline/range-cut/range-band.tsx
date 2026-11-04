// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { LIFTED } from "../lifted";
import { placed, type Span, type TimelineScale } from "../timeline-scale";

// The range the range tool draws across the strip, while it is drawn and
// once chosen. It is drawn across every lane (`className` sets its reach)
// for the pointer to pass through; in the row of beats and lulls an
// undrawn one (`isDrawn`) is where the chosen stretch is taken hold of
// (`onGrab`) and dragged to play elsewhere, drawn while it is carried.

export function RangeBand({
  className,
  isDrawn = true,
  isLifted = false,
  onGrab,
  scale,
  span,
}: {
  scale: TimelineScale;
  span: Span;
  /** How far up and down it reaches; the row it is in, inset, by default. */
  className?: string;
  /** Whether it is seen, rather than only taken hold of. */
  isDrawn?: boolean;
  /** Whether it is lifted to be moved, carried with the pointer. */
  isLifted?: boolean;
  /** Told a press on it, which may go on to drag it. */
  onGrab?: (event: React.PointerEvent) => void;
}) {
  return (
    <span
      aria-hidden
      className={cn(
        "absolute z-20 rounded-part",
        className ?? "inset-y-0.5",
        isDrawn && "bg-selected ring-2 ring-primary",
        onGrab ? "cursor-grab touch-none" : "pointer-events-none",
        isLifted && LIFTED,
      )}
      // Chosen, as only a stretch that can be taken hold of is, its island
      // stands over it (`use-editor-island-window.ts`).
      data-strip-island={onGrab ? "" : undefined}
      dir="ltr"
      onPointerDown={
        onGrab &&
        ((event) => {
          event.stopPropagation();
          onGrab(event);
        })
      }
      style={placed(scale, span.start, span.end)}
    />
  );
}
