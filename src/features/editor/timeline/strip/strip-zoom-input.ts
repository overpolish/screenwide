// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What zooms the strip and moves it by wheel. A pinch on the trackpad comes
// to WebKit, the editor's web view, as gesture events carrying how far the
// fingers have spread since the pinch began, not as wheel events, so both
// are read: the gesture's scale, and the wheel with ⌘ or Ctrl held, which a
// mouse sends and other engines send for a pinch. However many arrive
// between frames, the strip zooms once a frame by all of them together, so
// a long recording, which takes a while to draw, never falls behind the
// fingers.

/** How much a wheel step with ⌘ or Ctrl zooms per pixel. */
const ZOOM_PER_PX = 0.03;
/** The most a single wheel step counts for, in px: a mouse wheel's notch is
 * a large one, held to about half as much again; a pinch's steps are small
 * and pass whole. */
const MOST_STEP_PX = 13;
/** How many times over the strip zooms by a pinch's spread: fingers only
 * spread so far, and a long recording is many screens wide, so a pinch to
 * twice the distance zooms in eightfold. */
const PINCH_POWER = 3;

/** WebKit's pinch: `scale` is how far the fingers have spread since it
 * began, 1 where they started. */
type GestureEvent = UIEvent & { clientX: number; scale: number };

/** Listens on the scroll area `viewport` for zooming, telling `zoomAt` the
 * factor to scale the view's span by, under 1 to zoom in, and where across
 * the track it pivots, 0 to 1, `inset` px in from each side. An upright
 * wheel moves along the strip, which only scrolls sideways. Answers what
 * stops listening. */
export function listenToZoom(
  viewport: HTMLElement,
  inset: number,
  zoomAt: (factor: number, at: number) => void,
) {
  const across = (clientX: number) => {
    const box = viewport.getBoundingClientRect();
    return (clientX - box.left - inset) / Math.max(box.width - inset * 2, 1);
  };
  let factor = 1;
  let at = 0.5;
  let frame = 0;
  const zoomBy = (by: number, clientX: number) => {
    factor *= by;
    at = across(clientX);
    if (frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      const zoom = factor;
      factor = 1;
      if (zoom !== 1) zoomAt(zoom, at);
    });
  };

  // Between a pinch's start and end, a wheel with ⌘ or Ctrl held is the
  // same pinch told twice, and is left to the gesture.
  let pinchScale: number | null = null;
  const onGestureStart = (event: Event) => {
    event.preventDefault();
    pinchScale = (event as GestureEvent).scale;
  };
  const onGestureChange = (event: Event) => {
    event.preventDefault();
    const { clientX, scale } = event as GestureEvent;
    if (pinchScale === null || scale <= 0) return;
    // Spread apart, the fingers zoom in: the span grows narrower.
    zoomBy((pinchScale / scale) ** PINCH_POWER, clientX);
    pinchScale = scale;
  };
  const onGestureEnd = (event: Event) => {
    event.preventDefault();
    pinchScale = null;
  };
  const onWheel = (event: WheelEvent) => {
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      if (pinchScale !== null) return;
      const step = Math.min(
        Math.max(event.deltaY, -MOST_STEP_PX),
        MOST_STEP_PX,
      );
      zoomBy(Math.exp(step * ZOOM_PER_PX), event.clientX);
      return;
    }
    if (Math.abs(event.deltaY) > Math.abs(event.deltaX)) {
      event.preventDefault();
      viewport.scrollLeft += event.deltaY;
    }
  };
  viewport.addEventListener("wheel", onWheel, { passive: false });
  viewport.addEventListener("gesturestart", onGestureStart);
  viewport.addEventListener("gesturechange", onGestureChange);
  viewport.addEventListener("gestureend", onGestureEnd);
  return () => {
    cancelAnimationFrame(frame);
    viewport.removeEventListener("wheel", onWheel);
    viewport.removeEventListener("gesturestart", onGestureStart);
    viewport.removeEventListener("gesturechange", onGestureChange);
    viewport.removeEventListener("gestureend", onGestureEnd);
  };
}
