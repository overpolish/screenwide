// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// These are transient control feedback, including pseudo-element halos.
// Geometry, opacity, and keyframe animations retain their own lifecycle.
const INTERACTION_PROPERTIES = new Set([
  "background-color",
  "color",
  "border-top-color",
  "border-right-color",
  "border-bottom-color",
  "border-left-color",
  "outline-color",
  "box-shadow",
  "text-shadow",
  "fill",
  "stroke",
]);

function finishInteractionTransitions() {
  for (const animation of document.getAnimations()) {
    if (
      animation instanceof CSSTransition &&
      INTERACTION_PROPERTIES.has(animation.transitionProperty) &&
      animation.playState === "running" &&
      animation.playbackRate !== 0
    ) {
      animation.finish();
    }
  }
}

/**
 * Hidden WKWebViews can freeze a hover/press release transition at its first
 * frame. Finish that feedback at visibility boundaries so reopening any
 * window shows its current control state instead of resuming the old fade.
 */
export function installWindowInteractionTransitions() {
  let frame = 0;
  const cancelFrame = () => {
    if (frame) cancelAnimationFrame(frame);
    frame = 0;
  };
  const onVisibilityChange = () => {
    cancelFrame();
    finishInteractionTransitions();
    if (document.visibilityState === "visible") {
      // React can commit restored focus/hover state after visibilitychange.
      // Settle those transitions before the next paint too. Fresh input
      // cancels this pass so its feedback animates normally.
      frame = requestAnimationFrame(() => {
        frame = 0;
        finishInteractionTransitions();
      });
    }
  };
  document.addEventListener("visibilitychange", onVisibilityChange);
  document.addEventListener("pointerdown", cancelFrame, true);
  document.addEventListener("keydown", cancelFrame, true);
  document.addEventListener("pointermove", cancelFrame, true);
  return () => {
    cancelFrame();
    document.removeEventListener("visibilitychange", onVisibilityChange);
    document.removeEventListener("pointerdown", cancelFrame, true);
    document.removeEventListener("keydown", cancelFrame, true);
    document.removeEventListener("pointermove", cancelFrame, true);
  };
}
