// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  useEffect,
  useLayoutEffect,
  useReducer,
  useRef,
  useState,
} from "react";

import { type Playhead } from "../scrub-playhead";
import { clampView, type View } from "../timeline-scale";

import { stripViewNow } from "./strip-view-now";

// What part of the recording the strip shows. The strip scrolls in a
// ScrollArea, as every scrolling region does, so it fades where there is
// more to see and two fingers move it natively; the track inside is the
// whole recording at the view's zoom, and the scroll position and the view
// are kept one. A pinch, or the wheel with ⌘ or Ctrl, zooms round the
// pointer; the minimap's window moves and zooms it by being dragged.
// Nothing else moves the view, except to bring a new choice into it, or
// the playhead while it plays: choosing one thing never pushes the rest
// aside.
//
// Only the zoom is the strip's state. Where the view starts is kept apart
// (`startRef`, published as `now`): nothing on the track moves when it
// scrolls, so a pan scrolls the page the browser already drew rather than
// drawing the whole recording again for every scroll event, which a long
// recording could not keep up with. The zoom is kept exactly as it was set,
// so the track's width is the same number on every draw and nothing laid
// along it is written again for a difference in the last decimal place.

/** How wide a second is drawn when the view opens, at least, in px: a
 * short recording fits whole, a long one opens on its first minute or so. */
const OPENING_PX_PER_S = 16;
/** The fewest seconds the view zooms in to. */
const FEWEST_S = 3;
/** How much of the view a reveal leaves before what it reveals. */
const LEAD = 0.15;
/** Room at each end of the track, in px, so the playhead's knob at the very
 * start or end of the recording is drawn whole. */
const TRACK_INSET_PX = 8;
/** How much a pinch, or a wheel step with ⌘ or Ctrl, zooms per pixel. */
const ZOOM_PER_PX = 0.01;

export function useStripView({
  duration,
  focus,
  isPlaying,
  playhead,
  takenOut,
  width,
}: {
  /** The edited recording's length, in seconds. */
  duration: number;
  /** What is chosen, as shares of the edited recording, and a key naming
   * it: brought into view when the key changes; null with nothing chosen. */
  focus: (View & { key: string }) | null;
  isPlaying: boolean;
  playhead: Playhead;
  /** The seconds taken out of the recording, by cuts and the squeeze,
   * which the strip runs on past the edited recording's end to show. */
  takenOut: number;
  /** The scroll area's width, in px. */
  width: number;
}) {
  const least = Math.min(1, FEWEST_S / Math.max(duration, 1e-6));
  // Kept in played seconds rather than shares, and fixed once the strip is
  // first shown, so a trim, a cut or a squeeze changing how long the
  // recording plays leaves a second as wide as it was: the strip never
  // rescales under the pointer. What cuts and the squeeze take out stays on
  // the strip past the end, so the strip is always as long as the recording
  // and the gaps in it; a gap taken away shortens it, and the view is drawn
  // back inside it at once (below), as the next zoom or pan would.
  const [span, setSpan] = useState<number | null>(null);
  if (span === null && width > 0)
    setSpan(Math.min(width / OPENING_PX_PER_S, duration + takenOut));
  const seconds = span ?? duration + takenOut;
  const length = Math.max(duration, 1e-6);
  /** Where the strip ends, past the edited recording's end by what was
   * taken out, as a share of the edited recording; never short of its end,
   * so a recording with no length (a recovered one) still has a track. */
  const extent = Math.max(1, (duration + Math.max(takenOut, 0)) / length);
  const asked = seconds / length;
  const shown = Math.min(Math.max(asked, least), extent);
  const trackWidth =
    Math.max(0, width - TRACK_INSET_PX * 2) / Math.max(shown, asked);
  const [viewport, setViewport] = useState<HTMLElement | null>(null);
  const viewportRef = useRef<HTMLElement | null>(null);
  /** Where the view starts, in played seconds. */
  const startRef = useRef(0);
  /** Draws the strip again when a gap held at its start moves the view,
   * so the scroll lands with the gap's new width. */
  const [, redraw] = useReducer((draws: number) => draws + 1, 0);

  // Wheel and scroll events arrive faster than renders; each builds on the
  // last view rather than the one rendered.
  const latestRef = useRef({ asked, extent, least, length, shown, trackWidth });
  const viewNow = () => {
    const latest = latestRef.current;
    const start = startRef.current / latest.length;
    return clampView(
      { end: start + latest.shown, start },
      latest.least,
      latest.extent,
    );
  };
  const [now] = useState(() => stripViewNow(viewNow()));
  /** Whether the view was moved by something other than the scroll area
   * itself and is still to be scrolled to once the strip is drawn. */
  const pendingRef = useRef(true);
  /** Where the view started, in played seconds, when a gap's start was
   * taken hold of. */
  const heldStartRef = useRef<number | null>(null);

  const scrollToView = () => {
    const area = viewportRef.current;
    if (!area) return;
    const left = viewNow().start * latestRef.current.trackWidth;
    if (Math.abs(area.scrollLeft - left) > 0.5) area.scrollLeft = left;
  };
  const set = (next: View) => {
    const latest = latestRef.current;
    const kept = clampView(next, latest.least, latest.extent);
    startRef.current = kept.start * latest.length;
    const keptSpan = kept.end - kept.start;
    // A zoom draws the track again at its new width and scrolls after;
    // anything else keeps the span exactly and only scrolls.
    if (Math.abs(keptSpan - latest.asked) > 1e-9 * latest.asked) {
      latestRef.current = { ...latest, asked: keptSpan, shown: keptSpan };
      pendingRef.current = true;
      setSpan(keptSpan * latest.length);
    } else scrollToView();
    now.hold(viewNow());
  };
  const reveal = (start: number, end: number) => {
    const current = viewNow();
    if (start >= current.start && end <= current.end) return;
    const span = current.end - current.start;
    const from = start - span * LEAD;
    set({ end: from + span, start: from });
  };
  const zoomAt = (factor: number, at: number) => {
    const current = viewNow();
    const span = current.end - current.start;
    const next = Math.min(
      Math.max(span * factor, latestRef.current.least),
      latestRef.current.extent,
    );
    const pivot = current.start + at * span;
    set({ end: pivot - at * next + next, start: pivot - at * next });
  };
  const actionsRef = useRef({ reveal, set, zoomAt });

  // Before paint, once the track has its new width, so a zoom never shows a
  // frame scrolled to the old place. A zoom always scrolls, since the
  // position the scroll area holds is measured on the old width; a view
  // read from the scroll position never does, as the area has often
  // scrolled on by the time it is drawn, and putting it back would pull a
  // pan back.
  const scrolledWidthRef = useRef<number | null>(null);
  useLayoutEffect(() => {
    latestRef.current = { asked, extent, least, length, shown, trackWidth };
    actionsRef.current = { reveal, set, zoomAt };
    now.hold(viewNow());
    if (!viewport) return;
    const resized =
      scrolledWidthRef.current === null ||
      Math.abs(scrolledWidthRef.current - trackWidth) > 0.01;
    if (!resized && !pendingRef.current) return;
    scrolledWidthRef.current = trackWidth;
    pendingRef.current = false;
    scrollToView();
  });

  // A strip grown shorter than the view, as a gap taken away leaves it,
  // takes back the room past its end rather than leaving it empty.
  const total = duration + Math.max(takenOut, 0);
  useLayoutEffect(() => {
    if (span !== null && startRef.current + span > total + 1e-6)
      actionsRef.current.set(viewNow());
  }, [span, total]);

  useEffect(() => {
    if (!viewport) return;
    const onScroll = () => {
      const { length: playing, trackWidth: track } = latestRef.current;
      if (track <= 0) return;
      startRef.current = (viewport.scrollLeft / track) * playing;
      now.hold(viewNow());
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
  }, [now, viewport]);

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

  return {
    /** How wide, in px, the room past the track's end for what was taken
     * out is: as wide as it would play. */
    bandWidth: trackWidth * (extent - 1),
    /** The view shown this moment, which a pan changes without the strip
     * drawing again (`useStripViewNow`). */
    now,
    onInitialized: (area: HTMLElement | null) => {
      viewportRef.current = area;
      setViewport(area);
    },
    /** How wide a second is drawn, in px. */
    pxPerSecond: trackWidth / length,
    /** Shows `next`, as the minimap moves the view, scrolling to it. */
    set: (next: View) => {
      set(next);
    },
    /** Told null as a gap's start is taken hold of, then how many seconds
     * it has grown there since: the view moves by as much from where it
     * was, its zoom kept, so the start stays under the pointer and what
     * came before is pushed back. Measured from the hold, it cannot drift;
     * at the strip's start it can go no further back. It scrolls once the
     * strip is drawn with the gap's new width. */
    shiftSinceHold: (grown: number | null) => {
      if (grown === null) {
        heldStartRef.current = viewNow().start * latestRef.current.length;
        return;
      }
      const from = heldStartRef.current;
      if (from === null) return;
      startRef.current = Math.max(0, from + grown);
      pendingRef.current = true;
      redraw();
    },
    trackWidth,
  };
}
