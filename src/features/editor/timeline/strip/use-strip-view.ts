// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { type Playhead } from "../scrub-playhead";
import { clampView, type View } from "../timeline-scale";

// What part of the recording the strip shows. The strip scrolls in a
// ScrollArea, as every scrolling region does, so it fades where there is
// more to see and two fingers move it natively; the track inside is the
// whole recording at the view's zoom, and the scroll position and the view
// are kept one. A pinch, or the wheel with ⌘ or Ctrl, zooms round the
// pointer; the minimap's window moves and zooms it by being dragged.
// Nothing else moves the view, except to bring a new choice into it, or
// the playhead while it plays: choosing one thing never pushes the rest
// aside.

/** How wide a second is drawn when the view opens, at least, in px: a
 * short recording fits whole, a long one opens on its first minute or so. */
const OPENING_PX_PER_S = 16;
/** The fewest seconds the view zooms in to. */
const FEWEST_S = 3;
/** How much of the view a reveal leaves before what it reveals. */
const LEAD = 0.15;
/** Room at each end of the track, in px, so the playhead's knob at the very
 * start or end of the recording is drawn whole. */
export const TRACK_INSET_PX = 8;
/** How much a pinch, or a wheel step with ⌘ or Ctrl, zooms per pixel. */
const ZOOM_PER_PX = 0.01;

export function useStripView({
  duration,
  focus,
  isPlaying,
  playhead,
  width,
}: {
  /** The edited recording's length, in seconds. */
  duration: number;
  /** What is chosen, as shares of the edited recording, and a key naming
   * it: brought into view when the key changes; null with nothing chosen. */
  focus: (View & { key: string }) | null;
  isPlaying: boolean;
  playhead: Playhead;
  /** The scroll area's width, in px. */
  width: number;
}) {
  const least = Math.min(1, FEWEST_S / Math.max(duration, 1e-6));
  // Kept in played seconds rather than shares, and fixed once the strip is
  // first shown, so a trim, a cut or a squeeze changing how long the
  // recording plays leaves a second as wide as it was: the strip never
  // rescales under the pointer, and a recording grown shorter than the view
  // leaves room past its end rather than stretching to fill it.
  const [chosen, setChosen] = useState<View | null>(null);
  if (chosen === null && width > 0)
    setChosen({ end: Math.min(width / OPENING_PX_PER_S, duration), start: 0 });
  const seconds = chosen ?? { end: duration, start: 0 };
  const length = Math.max(duration, 1e-6);
  const view = clampView(
    { end: seconds.end / length, start: seconds.start / length },
    least,
  );
  const trackWidth =
    Math.max(0, width - TRACK_INSET_PX * 2) /
    Math.max(view.end - view.start, (seconds.end - seconds.start) / length);
  const [viewport, setViewport] = useState<HTMLElement | null>(null);

  // Wheel and scroll events arrive faster than renders; each builds on the
  // last view rather than the one rendered.
  const latestRef = useRef({ least, length, trackWidth, view });
  useLayoutEffect(() => {
    latestRef.current = { least, length, trackWidth, view };
  });
  const set = (next: View) => {
    const { least: fewest, length: playing } = latestRef.current;
    const kept = clampView(next, fewest);
    latestRef.current.view = kept;
    setChosen({ end: kept.end * playing, start: kept.start * playing });
  };
  const reveal = (start: number, end: number) => {
    const current = latestRef.current.view;
    if (start >= current.start && end <= current.end) return;
    const span = current.end - current.start;
    const from = start - span * LEAD;
    set({ end: from + span, start: from });
  };
  const zoomAt = (factor: number, at: number) => {
    const current = latestRef.current.view;
    const span = current.end - current.start;
    const next = Math.min(Math.max(span * factor, latestRef.current.least), 1);
    const pivot = current.start + at * span;
    set({ end: pivot - at * next + next, start: pivot - at * next });
  };
  const actionsRef = useRef({ reveal, set, zoomAt });
  useLayoutEffect(() => {
    actionsRef.current = { reveal, set, zoomAt };
  });

  // Before paint, once the track has its new width, so a zoom never shows a
  // frame scrolled to the old place.
  useLayoutEffect(() => {
    if (!viewport) return;
    const left = view.start * trackWidth;
    if (Math.abs(viewport.scrollLeft - left) > 0.5) viewport.scrollLeft = left;
  }, [viewport, view.start, trackWidth]);

  useEffect(() => {
    if (!viewport) return;
    const onScroll = () => {
      const { trackWidth: track, view: was } = latestRef.current;
      if (track <= 0) return;
      const start = viewport.scrollLeft / track;
      if (Math.abs(start - was.start) * track < 0.5) return;
      actionsRef.current.set({ end: start + was.end - was.start, start });
    };
    // An upright wheel moves along the strip, which only scrolls sideways.
    const onWheel = (event: WheelEvent) => {
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault();
        const box = viewport.getBoundingClientRect();
        const x =
          (event.clientX - box.left - TRACK_INSET_PX) /
          Math.max(box.width - TRACK_INSET_PX * 2, 1);
        // A pinch sends small steps; a mouse wheel's notch is a large one,
        // held to a gentle zoom.
        const step = Math.min(Math.max(event.deltaY, -40), 40);
        actionsRef.current.zoomAt(Math.exp(step * ZOOM_PER_PX), x);
        return;
      }
      if (Math.abs(event.deltaY) > Math.abs(event.deltaX)) {
        event.preventDefault();
        viewport.scrollLeft += event.deltaY;
      }
    };
    viewport.addEventListener("scroll", onScroll, { passive: true });
    viewport.addEventListener("wheel", onWheel, { passive: false });
    return () => {
      viewport.removeEventListener("scroll", onScroll);
      viewport.removeEventListener("wheel", onWheel);
    };
  }, [viewport]);

  // Only a new choice moves the view, never an edit to what is chosen.
  const focusKey = focus?.key ?? null;
  const focusRef = useRef(focus);
  useLayoutEffect(() => {
    focusRef.current = focus;
  });
  useEffect(() => {
    const chosenSpan = focusRef.current;
    if (chosenSpan) actionsRef.current.reveal(chosenSpan.start, chosenSpan.end);
  }, [focusKey, width]);

  useEffect(() => {
    if (!isPlaying) return;
    return playhead.subscribe((_, ratio) => {
      actionsRef.current.reveal(ratio, ratio);
    });
  }, [isPlaying, playhead]);

  return { onInitialized: setViewport, set, trackWidth, view };
}
