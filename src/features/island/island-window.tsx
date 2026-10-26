// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import {
  type ReactNode,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import { motionDurations } from "../../lib/motion";
import { useTauriEvent } from "../../lib/use-tauri-event";
import { ISLAND } from "../../primitives/island/island-geometry";

import type { IslandNotch } from "../../primitives/island/island-geometry";
import type { IslandPart } from "../../primitives/island/island-outline-sync";

// The island's window: one, whatever the island shows, as there is one
// notch. Rust brings the island out (`island://shown`, with the notch of the
// screen it is on, or where to hang by a box), asks it to tuck away
// (`island://hide`), after which the window goes, or moves it
// (`island://move`). Clicks pass through the window everywhere but where
// the island reports its parts to be.

/** What the island's window hands what the island shows. */
export type IslandHost = {
  isShown: boolean;
  notch: IslandNotch | null;
  onHidden: () => void;
  onShapeChange: (parts: IslandPart[]) => void;
  /** Which coming-out this is. It changes when the island moves, and an
   * island that can move keys itself on it, so it starts tucked away where
   * it now is instead of folding up where it was. */
  showing: number;
};

/** Where an island hanging by a box sits in its window, in px: its top, the
 * centre it is drawn round, and the span it stays within. */
type BoxAnchor = { centerX: number; maxX: number; minX: number; top: number };

type Shown = { anchor: BoxAnchor | null; notch: IslandNotch | null };
type Presence = Shown & { isShown: boolean };

const finishHiding = () => {
  void invoke("finish_hiding_island");
};

/** How far to move the island, centred in the window, so it is centred on
 * the box as far as it can be while staying within the span. */
const shiftFor = (anchor: BoxAnchor, width: number) => {
  const half = width / 2;
  const centre = Math.min(
    Math.max(anchor.centerX, anchor.minX + half),
    Math.max(anchor.maxX - half, anchor.minX + half),
  );
  return centre - window.innerWidth / 2;
};

export function IslandWindow({
  children,
}: {
  children: (island: IslandHost) => ReactNode;
}) {
  const [presence, setPresence] = useState<Presence>({
    anchor: null,
    isShown: false,
    notch: null,
  });
  const [showing, setShowing] = useState(0);

  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);
  const frameRef = useRef(0);
  const holderRef = useRef<HTMLDivElement>(null);
  const anchorRef = useRef<BoxAnchor | null>(null);
  const widthRef = useRef(0);
  const shiftRef = useRef(0);

  // Moved directly rather than through state: the island reports its shape
  // on every frame it changes, and the shift only needs its width. Answers
  // how far the island moved.
  const place = () => {
    const anchor = anchorRef.current;
    const shift = anchor ? shiftFor(anchor, widthRef.current) : 0;
    const moved = shift - shiftRef.current;
    shiftRef.current = shift;
    if (holderRef.current) {
      holderRef.current.style.transform = shift
        ? `translateX(${String(shift)}px)`
        : "";
    }
    return moved;
  };

  const reportShape = (parts: IslandPart[]) => {
    let moved = 0;
    if (parts.length > 0) {
      const start = Math.min(...parts.map((part) => part.x));
      const end = Math.max(...parts.map((part) => part.x + part.width));
      widthRef.current = end - start;
      moved = place();
    }
    // The parts were measured where the island was before it moved.
    const rects = moved
      ? parts.map((part) => ({ ...part, x: part.x + moved }))
      : parts;
    void invoke("set_island_hit_area", { rects });
  };

  useLayoutEffect(() => {
    anchorRef.current = presence.anchor;
    place();
  }, [presence.anchor]);

  // A move under way: the island tucks away where it is, then is laid out
  // afresh, hidden with no notch so nothing of it shows, and the window is
  // told it can move. `island://shown` then brings it out where it now is.
  const moveRef = useRef<number | null>(null);
  const tuckRef = useRef(0);
  const [readyToMove, setReadyToMove] = useState<number | null>(null);

  useEffect(() => {
    let disposed = false;
    void invoke<Presence>("island_presence").then((current) => {
      if (!disposed && !isToldRef.current) setPresence(current);
    });
    return () => {
      disposed = true;
      cancelAnimationFrame(frameRef.current);
      window.clearTimeout(tuckRef.current);
    };
  }, []);

  useTauriEvent("island://shown", (payload) => {
    isToldRef.current = true;
    const shown = payload as Shown;
    window.clearTimeout(tuckRef.current);
    moveRef.current = null;
    setPresence((current) => ({
      ...current,
      anchor: shown.anchor,
      notch: shown.notch,
    }));
    // Out a frame after the notch it comes out of is laid out, so it is seen
    // coming out of that notch.
    cancelAnimationFrame(frameRef.current);
    frameRef.current = requestAnimationFrame(() => {
      setPresence((current) => ({ ...current, isShown: true }));
    });
  });
  useTauriEvent("island://hide", () => {
    isToldRef.current = true;
    window.clearTimeout(tuckRef.current);
    moveRef.current = null;
    cancelAnimationFrame(frameRef.current);
    setPresence((current) => ({ ...current, isShown: false }));
  });
  useTauriEvent("island://move", (payload) => {
    isToldRef.current = true;
    const request = payload as number;
    moveRef.current = request;
    cancelAnimationFrame(frameRef.current);
    setPresence((current) => ({ ...current, isShown: false }));
    // Once its shape has tucked away, not waiting for a speck to fade.
    window.clearTimeout(tuckRef.current);
    tuckRef.current = window.setTimeout(() => {
      if (moveRef.current !== request) return;
      setPresence((current) => ({ ...current, notch: null }));
      setShowing((current) => current + 1);
      setReadyToMove(request);
    }, motionDurations.travel * 1000);
  });
  useEffect(() => {
    if (readyToMove === null) return;
    // Two frames: the first is drawn with nothing showing, and the second
    // only starts once that one is on screen.
    let frame = requestAnimationFrame(() => {
      frame = requestAnimationFrame(() => {
        void invoke("island_ready_to_move", { request: readyToMove });
      });
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [readyToMove]);

  const top = presence.anchor
    ? presence.anchor.top
    : presence.notch
      ? 0
      : ISLAND.screenGap;

  return (
    <div className="fixed inset-0">
      <div className="absolute inset-x-0" ref={holderRef} style={{ top }}>
        {children({
          isShown: presence.isShown,
          notch: presence.notch,
          onHidden: finishHiding,
          onShapeChange: reportShape,
          showing,
        })}
      </div>
    </div>
  );
}
