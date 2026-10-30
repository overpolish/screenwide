// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef, useState } from "react";

import { motionDurations } from "../../../../lib/motion";
import { type TimelineCut } from "../cut-seams/cut-seams";
import { type Span, timelineScale, type TimeMap } from "../timeline-scale";

import { type DropTarget, dropTargets } from "./strip-drop-targets";
import { heldPlace, landing, movedMap } from "./strip-move-preview";

// Moving stretches along the strip: a beat, a lull, what is chosen, or a
// stretch the scissors drew, is lifted and dragged, and dropped where it is
// to play. As the annotation lanes do with a held bar, the pointer moves
// only what is lifted, written straight to it as an offset (`--lift-x` on
// the row it is in); the strip is laid out again only when the place it
// would land changes, and glides there, so room opens where it will go and
// letting go keeps what is shown. Taken back where it was, it stays. A
// press that never travels is left to choose what it pressed.

/** How far, in px, a press travels before it moves rather than chooses, as
 * the strip's other drags do. */
const DRAG_SLOP_PX = 4;
/** How much nearer, in px, another place must be before what is lifted
 * goes there instead of where it is going. */
const STEADY_PX = 12;
/** How long, in ms, the pointer rests over a new place before the strip
 * makes room there, so sweeping past places never shuffles the row. */
const REST_MS = 150;

export function useStripReorder({
  beats,
  cuts,
  fractionOf,
  map,
  onMove,
  width,
}: {
  beats: Span[];
  cuts: TimelineCut[];
  fractionOf: (event: { clientX: number }) => number;
  map: TimeMap;
  /** Told stretches moved, in the order they play, to play together
   * before a recorded moment, or at the end for null. */
  onMove: (spans: Span[], before: number | null) => void;
  /** The track's width, in px. */
  width: number;
}) {
  const scale = timelineScale(map);
  // Only what lays the strip out is state: what is lifted and where it
  // lands. Where it is drawn follows the pointer outside React.
  const [drag, setDrag] = useState<{
    spans: Span[];
    target: DropTarget;
  } | null>(null);
  // The glide stays on a moment past letting go, while the strip settles.
  const [isSettling, setIsSettling] = useState(false);
  const rowRef = useRef<HTMLDivElement>(null);
  // Where what is lifted is drawn from, and where its room opens, as shares
  // of the strip; the row offsets it by the difference.
  const liftRef = useRef({ slot: 0, start: 0 });
  const latestRef = useRef({ fractionOf, onMove, width });
  useLayoutEffect(() => {
    latestRef.current = { fractionOf, onMove, width };
  });
  const writeLift = () => {
    const { slot, start } = liftRef.current;
    rowRef.current?.style.setProperty(
      "--lift-x",
      `${String((start - slot) * latestRef.current.width)}px`,
    );
  };
  // Laid out again, the room may open elsewhere; what is lifted keeps to
  // the pointer.
  useLayoutEffect(() => {
    if (drag) writeLift();
  });

  const grab = (event: React.PointerEvent, spans: Span[]) => {
    if (event.button !== 0) return;
    const pressX = event.clientX;
    const from = fractionOf(event);
    // Measured once: the strip under the drag stays as it was until it is
    // let go.
    const targets = dropTargets({ beats, cuts, map, scale, spans });
    // Where the strip shows it landing, and where its room opens there;
    // and a new place the pointer is over, waiting for it to rest.
    let target: DropTarget | null = null;
    let slot = 0;
    let pending: DropTarget | null = null;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const show = (next: { slot: number; target: DropTarget }) => {
      clearTimeout(timer);
      pending = null;
      target = next.target;
      slot = next.slot;
      liftRef.current.slot = slot;
      setDrag({ spans, target });
    };
    const onPointerMove = (moved: PointerEvent) => {
      if (!target && Math.abs(moved.clientX - pressX) < DRAG_SLOP_PX) return;
      const place = heldPlace({
        from,
        scale,
        spans,
        x: latestRef.current.fractionOf(moved),
      });
      const next = landing({
        current: pending ?? target,
        scale,
        spans,
        start: place.start,
        steady: STEADY_PX / Math.max(latestRef.current.width, 1),
        targets,
      });
      if (!next) return;
      liftRef.current = { slot, start: place.start };
      writeLift();
      if (!target) {
        show(next);
        return;
      }
      if (next.target === target) {
        clearTimeout(timer);
        pending = null;
        return;
      }
      if (next.target === pending) return;
      clearTimeout(timer);
      pending = next.target;
      timer = setTimeout(() => {
        show(next);
      }, REST_MS);
    };
    const onPointerUp = () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", onPointerUp);
      clearTimeout(timer);
      setDrag(null);
      // Let go before the strip made room, it still lands where the pointer
      // is, and the strip glides there.
      const landed = pending ?? target;
      if (!landed) return;
      // The click that ends a drag does not choose what it ends over.
      const swallow = (click: MouseEvent) => {
        click.stopPropagation();
      };
      window.addEventListener("click", swallow, { capture: true, once: true });
      setTimeout(() => {
        window.removeEventListener("click", swallow, { capture: true });
      });
      setIsSettling(true);
      setTimeout(() => {
        setIsSettling(false);
      }, motionDurations.rearrange * 1000);
      if (landed.isStay) return;
      latestRef.current.onMove(
        [...spans].sort(
          (first, second) => scale.toX(first.start) - scale.toX(second.start),
        ),
        landed.before,
      );
    };
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerUp);
  };

  const moving = drag && !drag.target.isStay ? drag : null;
  return {
    /** Where what is drawn splits while stretches are lifted, as the move
     * will split it: their ends, and where they land. */
    edges: moving
      ? [
          ...moving.spans.flatMap(({ end, start }) => [start, end]),
          ...(moving.target.before === null ? [] : [moving.target.before]),
        ]
      : [],
    grab,
    /** Whether the strip's parts glide to their places. */
    isGliding: drag !== null || isSettling,
    /** What is lifted, while it is: drawn at the pointer, off its place. */
    lifted: drag?.spans ?? null,
    /** How the recording plays as the strip shows it: with what is lifted
     * where it would land. */
    map: moving ? movedMap(map, moving.spans, moving.target.played) : map,
    /** The row what is lifted is drawn in, which carries its offset. */
    rowRef,
  };
}
