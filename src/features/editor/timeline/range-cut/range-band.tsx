// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { LIFTED } from "../lifted";
import { placed, type Span, type TimelineScale } from "../timeline-scale";

// The stretch the scissors draw across the strip, over lulls and beats
// alike, while it is drawn and once chosen. Given `onGrab`, the chosen
// stretch can be taken hold of and dragged to play elsewhere.

export function RangeBand({
  isLifted = false,
  onGrab,
  scale,
  span,
}: {
  scale: TimelineScale;
  span: Span;
  /** Whether it is lifted to be moved, carried with the pointer. */
  isLifted?: boolean;
  /** Told a press on it, which may go on to drag it. */
  onGrab?: (event: React.PointerEvent) => void;
}) {
  return (
    <span
      aria-hidden
      className={cn(
        "absolute inset-y-0.5 z-20 rounded-part bg-selected ring-2 ring-primary",
        onGrab ? "cursor-grab touch-none" : "pointer-events-none",
        isLifted && LIFTED,
      )}
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
