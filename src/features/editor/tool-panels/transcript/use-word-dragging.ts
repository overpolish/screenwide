// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useEffect, useRef, useState } from "react";

import { DropBoundary, dropBoundary } from "./transcript-drop-geometry";
import { ClientPoint } from "./transcript-edit-span";
import { WordSelection } from "./transcript-units";

/** How far a press travels before it carries the words rather than clicks. */
const DRAG_SLOP_PX = 4;
/** How close to the top or bottom of the transcript's scroll area the
 * pointer comes before the words scroll under it, and how far they scroll
 * each frame at the very edge. */
const SCROLL_EDGE_PX = 24;
const SCROLL_STEP_PX = 12;

/** Words carried by a drag: the chosen span, and where they would land,
 * null where letting go now leaves them where they are. */
export type WordDrag = { boundary: DropBoundary | null; span: WordSelection };

const isRtl = (element: HTMLElement | undefined) =>
  element !== undefined && getComputedStyle(element).direction === "rtl";

/** The element `element` scrolls in, or null where nothing scrolls it. */
const scrollerOf = (element: HTMLElement | null) => {
  for (let at = element?.parentElement; at; at = at.parentElement) {
    const { overflowY } = getComputedStyle(at);
    if (overflowY === "auto" || overflowY === "scroll") return at;
  }
  return null;
};

/**
 * Dragging the chosen words to another place in the transcript, the way
 * chosen text is dragged: a press inside the choice that travels a few
 * pixels carries it, a caret marks the word boundary it would land at, and
 * letting go there moves it. A press that does not travel is a click, which
 * `onClick` makes on release. Near the top or bottom of the scroll area the
 * words scroll under the pointer. Escape, or the pointer going away, puts
 * everything back.
 */
export function useWordDragging({
  onMove,
  rootRef,
  spansRef,
}: {
  /** Move the units in `span` to stand before unit `before`, the unit count
   * for after the last. */
  onMove: (span: WordSelection, before: number) => void;
  rootRef: RefObject<HTMLElement | null>;
  spansRef: RefObject<HTMLElement[]>;
}) {
  const [drag, setDrag] = useState<WordDrag | null>(null);
  const stopRef = useRef<() => void>(() => undefined);
  useEffect(
    () => () => {
      stopRef.current();
    },
    [],
  );

  const begin = (
    start: ClientPoint,
    span: WordSelection,
    onClick: () => void,
  ) => {
    stopRef.current();
    const scroller = scrollerOf(rootRef.current);
    let point = start;
    let moved = false;
    let boundary: DropBoundary | null = null;
    let frame = 0;
    const measure = () => {
      const spans = spansRef.current;
      const found = dropBoundary(
        spans.map((element) => element.getBoundingClientRect()),
        point,
        (index) => isRtl(spans[index]),
      );
      // Landing inside the words, or at either of their edges, leaves them
      // where they are.
      boundary =
        found && (found.before < span.first || found.before > span.last + 1)
          ? found
          : null;
      setDrag((current) =>
        current?.boundary?.before === boundary?.before &&
        current?.boundary?.beside === boundary?.beside
          ? current
          : { boundary, span },
      );
    };
    const scroll = () => {
      frame = requestAnimationFrame(scroll);
      if (!scroller) return;
      const bounds = scroller.getBoundingClientRect();
      const step = (reach: number) =>
        Math.ceil(SCROLL_STEP_PX * Math.min(1, reach / SCROLL_EDGE_PX));
      const above = bounds.top + SCROLL_EDGE_PX - point.y;
      const below = point.y - (bounds.bottom - SCROLL_EDGE_PX);
      const by = above > 0 ? -step(above) : below > 0 ? step(below) : 0;
      if (by === 0) return;
      const before = scroller.scrollTop;
      scroller.scrollTop += by;
      if (scroller.scrollTop !== before) measure();
    };
    const onPointerMove = (event: PointerEvent) => {
      point = { x: event.clientX, y: event.clientY };
      if (
        !moved &&
        Math.hypot(point.x - start.x, point.y - start.y) < DRAG_SLOP_PX
      )
        return;
      if (!moved) {
        moved = true;
        frame = requestAnimationFrame(scroll);
      }
      measure();
    };
    const finish = (letGo: boolean) => {
      stopRef.current();
      if (!moved) {
        if (letGo) onClick();
        return;
      }
      if (letGo && boundary) onMove(span, boundary.before);
    };
    const onPointerUp = () => {
      finish(true);
    };
    const onCancel = () => {
      finish(false);
    };
    // Escape belongs to the drag while it lasts, not to the transcript,
    // which would let the choice go.
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      finish(false);
    };
    stopRef.current = () => {
      stopRef.current = () => undefined;
      cancelAnimationFrame(frame);
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", onCancel);
      window.removeEventListener("blur", onCancel);
      window.removeEventListener("keydown", onKeyDown, true);
      setDrag(null);
    };
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onCancel);
    window.addEventListener("blur", onCancel);
    window.addEventListener("keydown", onKeyDown, true);
  };

  return { begin, drag };
}
