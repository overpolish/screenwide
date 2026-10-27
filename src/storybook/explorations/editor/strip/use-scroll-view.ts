// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef, useState } from "react";

import { type ViewControls } from "./use-view";

// The strip scrolls in a ScrollArea, as every scrolling region does, so it
// fades where there is more to see and two fingers move it natively. The
// track inside is the whole recording at the view's zoom, and the scroll
// position and the view are kept one: scrolling moves the view, and the
// view, moved by the overview, a zoom or a reveal, scrolls the strip.

/** Room at each end of the track, in px, so a knob at the very start or
 * end of the recording is drawn whole inside the scroll area. */
const TRACK_INSET_PX = 8;
/** How much a pinch, or a wheel step with ⌘ or Ctrl, zooms per pixel. */
const ZOOM_PER_PX = 0.01;

export function useScrollView(controls: ViewControls, frameWidth: number) {
  const { view } = controls;
  const trackWidth =
    Math.max(0, frameWidth - TRACK_INSET_PX * 2) / (view.end - view.start);
  const [viewport, setViewport] = useState<HTMLElement | null>(null);
  const latestRef = useRef({ controls, trackWidth });
  useLayoutEffect(() => {
    latestRef.current = { controls, trackWidth };
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
      const { controls: current, trackWidth: width } = latestRef.current;
      if (width <= 0) return;
      const start = viewport.scrollLeft / width;
      const { end, start: was } = current.view;
      if (Math.abs(start - was) * width < 0.5) return;
      current.set({ end: start + end - was, start });
    };
    // A pinch, or the wheel with ⌘ or Ctrl, zooms round the pointer; an
    // upright wheel moves along the strip, which only scrolls sideways.
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
        latestRef.current.controls.zoomAt(Math.exp(step * ZOOM_PER_PX), x);
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

  return { onInitialized: setViewport, trackWidth };
}
