// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A gap dragged along the strip slides: where it starts follows the pointer
// in played time, splitting the recording wherever it is let go, rather than
// being lifted and dropped between beats as a beat is moved. A press that
// never travels is left to choose it. Its start dragged lengthens or
// shortens it as its end does, and the view moves by as much, so the start
// stays under the pointer and what came before is pushed back.

/** How far, in px, a press travels before it slides rather than chooses. */
const SLIDE_SLOP_PX = 4;

/** Slides a gap from the press `event`, its start `from` played seconds in,
 * telling `onSlide` where it starts as the pointer goes. */
export function slideGap(
  event: React.PointerEvent,
  {
    from,
    onSlide,
    pxPerSecond,
  }: {
    from: number;
    onSlide: (at: number) => void;
    /** How wide a second is drawn, in px. */
    pxPerSecond: number;
  },
) {
  const x = event.clientX;
  let isSliding = false;
  const move = (moved: PointerEvent) => {
    const dx = moved.clientX - x;
    if (!isSliding && Math.abs(dx) < SLIDE_SLOP_PX) return;
    isSliding = true;
    onSlide(Math.max(0, from + dx / Math.max(pxPerSecond, 1e-6)));
  };
  const end = () => {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", end);
    window.removeEventListener("pointercancel", end);
  };
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", end);
  window.addEventListener("pointercancel", end);
}
