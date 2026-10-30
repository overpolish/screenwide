// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { cn } from "../../../../lib/styling";
import { type Playhead } from "../scrub-playhead";
import { type View, WHOLE } from "../timeline-scale";

// Where you are: a line down the strip in the primary, with a knob along the
// top, in a band of its own above the scenes so it is never taken for part
// of one. Only the knob is pressed, and given `onScrub`, dragged; its hit
// area is larger than the dot drawn. Its place is written straight to the
// line as the playhead moves, so playing never renders it.

export function TimelinePlayhead({
  onScrub,
  playhead,
  view = WHOLE,
}: {
  /** Where the playhead is, as its share of the edited recording. */
  playhead: Playhead;
  /** Told each pointer event of a drag on the knob. */
  onScrub?: (event: React.PointerEvent) => void;
  /** The part of the edited recording the strip under it draws. */
  view?: View;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const { end, start } = view;
  useEffect(
    () =>
      playhead.subscribe((_, ratio) => {
        if (!ref.current) return;
        const x = (ratio - start) / Math.max(end - start, 1e-6);
        ref.current.style.insetInlineStart = `${String(x * 100)}%`;
      }),
    [end, playhead, start],
  );
  return (
    // Drawn for the pointer: the transport reads the position out.
    <span
      aria-hidden
      className="pointer-events-none absolute inset-y-0 z-10 w-0.5 -translate-x-1/2 bg-primary"
      dir="ltr"
      ref={ref}
    >
      <span
        aria-hidden
        className={cn(
          "absolute -top-3 start-1/2 flex size-6 -translate-x-1/2 items-center justify-center",
          onScrub && "pointer-events-auto cursor-ew-resize touch-none",
        )}
        onPointerDown={(event) => {
          if (!onScrub) return;
          event.stopPropagation();
          event.currentTarget.setPointerCapture(event.pointerId);
          onScrub(event);
        }}
        onPointerMove={(event) => {
          if (onScrub && event.currentTarget.hasPointerCapture(event.pointerId))
            onScrub(event);
        }}
      >
        <span className="size-3 rounded-full bg-primary" />
      </span>
    </span>
  );
}
