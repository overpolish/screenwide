// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect, useRef, useState } from "react";

import { type ScreenPoint } from "../word-hits";

import { type CuePlace } from "./place-cue";

/** How far a press moves before it is a drag rather than a click, in
 * points. */
const DRAG_SLOP = 4;

/** A cue being dragged: which, the word it would land on, and how far down
 * the screen the pointer holding its mark is. */
export type CueDrag = CuePlace & { clientY: number };

/**
 * Dragging a cue's mark to another word: from a press on the mark, the
 * pointer is followed over the whole window until it is let go, with the
 * word the cue `id` would land on (`landingAt`, from a point on screen;
 * undefined keeps the last). Let go, the cue is moved there (`onMove`); let
 * go without having moved, the press was a click (`onClick`).
 */
export function useCueDrag({
  landingAt,
  onMove,
}: {
  landingAt: (point: ScreenPoint, id: string) => number | undefined;
  onMove: (id: string, at: number) => void;
}) {
  const [drag, setDrag] = useState<CueDrag | null>(null);
  const stopRef = useRef<(() => void) | null>(null);
  const callbacksRef = useRef({ landingAt, onMove });
  callbacksRef.current = { landingAt, onMove };

  useEffect(
    () => () => {
      stopRef.current?.();
    },
    [],
  );

  /** Follows the pointer from a press at `press` on the mark of `cue`. */
  const begin = (cue: CuePlace, press: ScreenPoint, onClick: () => void) => {
    stopRef.current?.();
    let landing: CuePlace | null = null;
    const onPointerMove = (event: PointerEvent) => {
      // Let go where the page did not hear it, outside the window or as
      // the window was brought forward, the press is over: kept where it
      // was seen to land, and never taken for a click.
      if ((event.buttons & 1) === 0) {
        stop();
        if (landing) callbacksRef.current.onMove(cue.id, landing.at);
        return;
      }
      const moved = Math.hypot(
        event.clientX - press.clientX,
        event.clientY - press.clientY,
      );
      if (!landing && moved < DRAG_SLOP) return;
      const to =
        callbacksRef.current.landingAt(event, cue.id) ?? landing?.at ?? cue.at;
      landing = { at: to, id: cue.id };
      setDrag({ ...landing, clientY: event.clientY });
    };
    const onPointerUp = (event: PointerEvent) => {
      if (event.button !== 0) return;
      stop();
      if (landing) callbacksRef.current.onMove(cue.id, landing.at);
      else onClick();
    };
    const stop = () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", stop);
      stopRef.current = null;
      setDrag(null);
    };
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", stop);
    stopRef.current = stop;
  };

  return { begin, drag };
}
