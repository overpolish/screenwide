// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { type View } from "./strip-scale";

// What part of the recording the strip shows. Two fingers along the strip,
// or the wheel, move through it; pinching, or the wheel with ⌘ or Ctrl,
// zooms around the pointer; the window on the overview does both by being
// dragged. Nothing else moves the view, except to bring what was just
// chosen, or the playhead while it plays, into it: choosing one thing never
// pushes the rest aside.

/** How wide a second of recording is drawn when the view opens, at least:
 * a short recording fits whole, a long one opens on its first minute or so. */
const OPENING_PX_PER_S = 16;
/** The fewest seconds the view can be zoomed in to. */
const FEWEST_S = 3;
/** How much of the view a fresh reveal leaves before what it reveals. */
const LEAD = 0.15;

/** `view` kept whole and inside the recording, no narrower than `least`. */
function clamped(view: View, least: number): View {
  const span = Math.min(Math.max(view.end - view.start, least), 1);
  const start = Math.min(Math.max(view.start, 0), 1 - span);
  return { end: start + span, start };
}

export function useView({
  duration,
  width,
}: {
  /** The edited recording's length, in seconds. */
  duration: number;
  /** The strip's width, in px. */
  width: number;
}) {
  const opening: View = {
    end: width > 0 ? Math.min(1, width / OPENING_PX_PER_S / duration) : 1,
    start: 0,
  };
  const least = Math.min(1, FEWEST_S / Math.max(duration, 1e-6));
  const [chosen, setChosen] = useState<View | null>(null);
  const view = clamped(chosen ?? opening, least);
  // Wheel events arrive faster than renders; each builds on the last.
  const viewRef = useRef(view);
  useEffect(() => {
    viewRef.current = view;
  });

  const set = (next: View) => {
    viewRef.current = clamped(next, least);
    setChosen(viewRef.current);
  };
  return {
    /** Moves the view by a share of the recording. */
    pan: (by: number) => {
      const current = viewRef.current;
      set({ end: current.end + by, start: current.start + by });
    },
    /** Shows the share from `start` to `end`, if it is not in view. */
    reveal: (start: number, end: number) => {
      const current = viewRef.current;
      if (start >= current.start && end <= current.end) return;
      const span = current.end - current.start;
      const from = start - span * LEAD;
      set({ end: from + span, start: from });
    },
    set,
    view,
    /** Zooms by `factor`, keeping what is under `at` (0 to 1 across the
     * strip) where it is. */
    zoomAt: (factor: number, at: number) => {
      const current = viewRef.current;
      const span = current.end - current.start;
      const next = Math.min(Math.max(span * factor, least), 1);
      const pivot = current.start + at * span;
      set({ end: pivot - at * next + next, start: pivot - at * next });
    },
  };
}

export type ViewControls = ReturnType<typeof useView>;
