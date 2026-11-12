// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";
import { flushSync } from "react-dom";

import { type Span, type TimelineScale } from "../timeline-scale";

// What holding a bar in a stack does. A press chooses it and takes hold
// at once. Dragged, the bar moves in time and, where it sits in a lane, up
// and down through the lanes too, which the lane follows (`onHold`); a
// chosen bar's end moves one side in time only. Up and down is measured in
// the lanes themselves, not on screen, so a bar stays under the pointer
// while they scroll under it, and a scroll with the pointer still applies
// as a move would. While ⌘ (Ctrl on Windows) is held, an end, or either end
// of a bar moved whole, snaps to what is in reach (`snap`). While Shift is
// held, the drag keeps to whichever way it has gone further from where it
// was pressed. Either key pressed or let go applies at once, without
// waiting for the pointer to move.

/** How far, in px, a press moves before it is a drag, so a press that
 * only chooses the bar never nudges it. */
const DRAG_PX = 3;

/** What a press takes hold of: the whole bar, or a chosen bar's end. */
export type BarPart = "body" | "end" | "start";

/** Where a dragged end settles: on something within reach, or where it was
 * let go. */
export type Snap = (seconds: number) => { isSnapped: boolean; seconds: number };

/** The keys held through a drag that change what it does. */
type DragKeys = { isShift: boolean; isSnap: boolean };

/** ⌘ on a Mac and Ctrl on Windows snap; the other is never held for it. */
const keysOf = (event: {
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
}): DragKeys => ({
  isShift: event.shiftKey,
  isSnap: event.metaKey || event.ctrlKey,
});

const DRAG_KEYS = new Set(["Control", "Meta", "Shift"]);

const NO_SNAP: Snap = (seconds) => ({ isSnapped: false, seconds });

type Hold = Span & {
  bar: Element;
  /** The recorded moment under the pointer when pressed. */
  grab: number;
  /** Whether the press has moved far enough to be a drag. */
  isDragging: boolean;
  /** Where the pointer last was, on screen, and the keys then held, for a
   * key or a scroll to apply to between moves. */
  last: { keys: DragKeys; x: number; y: number };
  part: BarPart;
  release: () => void;
  /** Where it was pressed: `x` on screen, `y` in the lanes. */
  x: number;
  y: number;
};

/** A drag in progress, as the lane needs it: where the pointer is in the
 * lanes, in px from their top however they are scrolled, against where it
 * was pressed, whether the bar is carried up and down with it or kept to
 * its lane, where in time it is going, and the moment an end has snapped
 * to, which the lane draws a guide down to. */
export type BarHold = {
  guide: number | null;
  isLifted: boolean;
  span: Span;
  startY: number;
  y: number;
};

/** Where `clientY` is in the lanes the bar is placed in, in px from their
 * top, however far they are scrolled. */
const laneY = (clientY: number, bar: Element) =>
  clientY - (bar.parentElement?.getBoundingClientRect().top ?? 0);

export function useStackBarDrag({
  isSelected,
  movesInTimeOnly = false,
  onHold,
  onRetime,
  onSelect,
  scale,
  snap = NO_SNAP,
  span,
}: {
  isSelected: boolean;
  /** Told each move of a drag, and null when it is let go; without it,
   * the bar only moves in time. */
  onHold: ((hold: BarHold | null) => void) | undefined;
  /** Told the bar's new span, and what of it was held: the whole bar or
   * one end. */
  onRetime: (start: number, end: number, part: BarPart) => void;
  onSelect: () => void;
  scale: TimelineScale;
  snap: Snap | undefined;
  span: Span;
  /** Never carried up and down, where its lanes only keep what overlaps
   * apart and its order means nothing. */
  movesInTimeOnly?: boolean;
}) {
  const holdRef = useRef<Hold | null>(null);

  /** The recorded moment under `clientX`, measured against the strip the
   * bar is placed in. */
  const secondsAt = (clientX: number, bar: Element) => {
    const box = bar.parentElement?.getBoundingClientRect();
    if (!box || box.width === 0) return 0;
    return scale.toSeconds((clientX - box.left) / box.width);
  };
  // Held on a snapped edge, the pointer keeps moving while the bar stays
  // put; only a change is told.
  const retime = (start: number, end: number, part: BarPart) => {
    if (start === span.start && end === span.end) return;
    onRetime(start, end, part);
  };

  /** Moves the held bar for the pointer at `x`, `y`, as the keys held say. */
  const apply = (x: number, y: number, keys: DragKeys) => {
    const held = holdRef.current;
    if (!held) return;
    held.last = { keys, x, y };
    const pointerY = laneY(y, held.bar);
    const dx = x - held.x;
    const dy = pointerY - held.y;
    if (!held.isDragging && Math.max(Math.abs(dx), Math.abs(dy)) < DRAG_PX)
      return;
    held.isDragging = true;
    const pointer = secondsAt(x, held.bar);
    const lock =
      held.part !== "body" || movesInTimeOnly
        ? "time"
        : keys.isShift
          ? Math.abs(dx) >= Math.abs(dy)
            ? "time"
            : "lane"
          : null;
    const { guide, span: next } =
      lock === "lane"
        ? { guide: null, span: { end: held.end, start: held.start } }
        : moveTo(held, pointer, keys.isSnap ? snap : NO_SNAP);
    onHold?.({
      guide,
      isLifted: held.part === "body" && lock !== "time",
      span: next,
      startY: held.y,
      y: pointerY,
    });
    retime(next.start, next.end, held.part);
  };
  // A key pressed or let go, or the lanes or the strip scrolled, between
  // moves applies through the latest render's handlers, as a move would.
  const applyRef = useRef(apply);
  useEffect(() => {
    applyRef.current = apply;
  });

  const grab = (
    event: React.PointerEvent<HTMLElement>,
    bar: Element,
    part: BarPart,
  ) => {
    event.currentTarget.setPointerCapture(event.pointerId);
    if (!isSelected) onSelect();
    const onKey = (key: KeyboardEvent) => {
      const held = holdRef.current;
      if (!DRAG_KEYS.has(key.key) || !held?.isDragging) return;
      applyRef.current(held.last.x, held.last.y, keysOf(key));
    };
    // A scroll does not bubble, but every one passes the window on its way
    // down. The bar is drawn again at once, in the frame the lanes moved in.
    const onScroll = () => {
      const held = holdRef.current;
      if (!held?.isDragging) return;
      // eslint-disable-next-line @eslint-react/dom-no-flush-sync -- left to React, a scroll's render lands a frame or two after the lanes moved, and the carried bar slips from under the pointer while they scroll.
      flushSync(() => {
        applyRef.current(held.last.x, held.last.y, held.last.keys);
      });
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("keyup", onKey);
    window.addEventListener("scroll", onScroll, {
      capture: true,
      passive: true,
    });
    holdRef.current = {
      bar,
      end: span.end,
      grab: secondsAt(event.clientX, bar),
      isDragging: false,
      last: { keys: keysOf(event), x: event.clientX, y: event.clientY },
      part,
      release: () => {
        window.removeEventListener("keydown", onKey);
        window.removeEventListener("keyup", onKey);
        window.removeEventListener("scroll", onScroll, { capture: true });
      },
      start: span.start,
      x: event.clientX,
      y: laneY(event.clientY, bar),
    };
  };

  const drag = (event: React.PointerEvent<HTMLElement>) => {
    apply(event.clientX, event.clientY, keysOf(event));
  };

  const release = () => {
    const held = holdRef.current;
    holdRef.current = null;
    if (!held) return;
    held.release();
    if (held.isDragging) onHold?.(null);
  };

  // A bar taken away mid-drag still lets go of the keys.
  useEffect(
    () => () => {
      holdRef.current?.release();
    },
    [],
  );

  return { drag, grab, release };
}

/** Where the held bar goes in time with the pointer at `pointer` seconds,
 * and the event an end has snapped to, if any. Moved whole, whichever end
 * comes near an event snaps to it; held by an end, that end alone moves. */
function moveTo(held: Hold, pointer: number, snap: Snap) {
  if (held.part !== "body") {
    const edge = snap(pointer);
    return {
      guide: edge.isSnapped ? edge.seconds : null,
      span:
        held.part === "start"
          ? { end: held.end, start: edge.seconds }
          : { end: edge.seconds, start: held.start },
    };
  }
  const length = held.end - held.start;
  const moved = held.start + pointer - held.grab;
  const head = snap(moved);
  const tail = snap(moved + length);
  const start = head.isSnapped
    ? head.seconds
    : tail.isSnapped
      ? tail.seconds - length
      : moved;
  return {
    guide: head.isSnapped ? head.seconds : tail.isSnapped ? tail.seconds : null,
    span: { end: start + length, start },
  };
}
