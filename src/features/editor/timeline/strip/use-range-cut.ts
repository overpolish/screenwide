// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { type Span } from "../timeline-scale";

// The scissors work anywhere on the strip while ⌥ is held. A blade follows
// the pointer, so what a press will do shows before it is made; dragged, it
// draws a stretch over lulls and beats alike, which is chosen when let go,
// to be cut from its island. A chosen lull's end keeps ⌥ for its own trim,
// so a press there is left to it.

/** The shortest stretch, in seconds, a drag chooses; less is a slip. */
const LEAST_S = 0.1;
/** How far, in px, a press travels before it draws rather than splits, as
 * the strip's other drags do. */
const PRESS_PX = 4;

export function useRangeCut({
  fractionOf,
  isCutting,
  onChoose,
  onSplit,
  secondsAt,
}: {
  /** Where the pointer is, as a share of the track's width. */
  fractionOf: (event: React.PointerEvent) => number;
  /** Whether the scissors are in hand. */
  isCutting: boolean;
  /** Told the stretch a drag drew, once let go. */
  onChoose: (span: Span) => void;
  /** Told a press that drew nothing, to split at the recorded moment. */
  onSplit: (at: number) => void;
  /** The recorded moment under the pointer. */
  secondsAt: (event: React.PointerEvent) => number;
}) {
  const fromRef = useRef<number | null>(null);
  const pressXRef = useRef(0);
  // Set by a drag, so the click that ends it does not choose what is under
  // the pointer as well.
  const wasDrawnRef = useRef(false);
  const [draft, setDraft] = useState<Span | null>(null);
  const [bladeX, setBladeX] = useState<number | null>(null);

  const spanTo = (event: React.PointerEvent): Span => {
    const from = fromRef.current ?? 0;
    const to = secondsAt(event);
    return { end: Math.max(from, to), start: Math.min(from, to) };
  };

  return {
    bladeX: isCutting ? bladeX : null,
    draft,
    handlers: {
      onClickCapture: (event: React.MouseEvent) => {
        if (!wasDrawnRef.current) return;
        wasDrawnRef.current = false;
        event.stopPropagation();
      },
      onPointerDownCapture: (event: React.PointerEvent<HTMLElement>) => {
        const target = event.target as Element;
        if (!isCutting || target.closest("[data-lull-end]")) return;
        event.stopPropagation();
        event.currentTarget.setPointerCapture(event.pointerId);
        fromRef.current = secondsAt(event);
        pressXRef.current = event.clientX;
        wasDrawnRef.current = true;
        setDraft(spanTo(event));
      },
      onPointerLeave: () => {
        setBladeX(null);
      },
      onPointerMoveCapture: (event: React.PointerEvent) => {
        setBladeX(fractionOf(event));
        if (fromRef.current === null) return;
        event.stopPropagation();
        setDraft(spanTo(event));
      },
      onPointerUpCapture: (event: React.PointerEvent) => {
        const from = fromRef.current;
        if (from === null) return;
        event.stopPropagation();
        const span = spanTo(event);
        fromRef.current = null;
        setDraft(null);
        if (Math.abs(event.clientX - pressXRef.current) < PRESS_PX)
          onSplit(from);
        else if (span.end - span.start >= LEAST_S) onChoose(span);
      },
    },
  };
}
