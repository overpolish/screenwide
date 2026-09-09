// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { installWindowInteractionTransitions } from "./window-interaction-transitions";

class Transition {
  finish = vi.fn();
  playState = "running";
  playbackRate = 1;
  constructor(public transitionProperty: string) {}
}

describe("window interaction transitions", () => {
  let animations: (Transition | { finish: ReturnType<typeof vi.fn> })[];
  let page: EventTarget & {
    getAnimations: () => typeof animations;
    visibilityState: string;
  };
  let stop: () => void;
  let nextFrame: (() => void) | undefined;

  beforeEach(() => {
    animations = [];
    nextFrame = undefined;
    page = Object.assign(new EventTarget(), {
      getAnimations: () => animations,
      visibilityState: "visible",
    });
    vi.stubGlobal("document", page);
    vi.stubGlobal("CSSTransition", Transition);
    vi.stubGlobal("requestAnimationFrame", (callback: () => void) => {
      nextFrame = callback;
      return 1;
    });
    vi.stubGlobal("cancelAnimationFrame", () => {
      nextFrame = undefined;
    });
    stop = installWindowInteractionTransitions();
  });

  afterEach(() => {
    stop();
    vi.unstubAllGlobals();
  });

  function visibility(value: string) {
    page.visibilityState = value;
    page.dispatchEvent(new Event("visibilitychange"));
  }

  it("settles frozen feedback across different control styles when hidden", () => {
    const feedback = [
      "background-color",
      "color",
      "box-shadow",
      "border-top-color",
      "fill",
    ].map((property) => new Transition(property));
    animations.push(...feedback);
    visibility("hidden");
    for (const transition of feedback)
      expect(transition.finish).toHaveBeenCalledOnce();
    expect(nextFrame).toBeUndefined();
  });

  it("settles feedback created by restored state before the first visible paint", () => {
    visibility("hidden");
    visibility("visible");
    const restoredHover = new Transition("background-color");
    animations.push(restoredHover);
    nextFrame?.();
    expect(restoredHover.finish).toHaveBeenCalledOnce();
  });

  it("preserves geometry, opacity, keyframes, and deliberately paused animations", () => {
    const paused = new Transition("background-color");
    paused.playState = "paused";
    const stopped = new Transition("color");
    stopped.playbackRate = 0;
    animations.push(
      new Transition("transform"),
      new Transition("opacity"),
      { finish: vi.fn() },
      paused,
      stopped,
    );
    visibility("hidden");
    visibility("visible");
    nextFrame?.();
    for (const animation of animations)
      expect(animation.finish).not.toHaveBeenCalled();
  });

  it.each(["pointerdown", "pointermove", "keydown"])(
    "preserves fresh %s feedback after reopening",
    (type) => {
      visibility("visible");
      const freshFeedback = new Transition("background-color");
      animations.push(freshFeedback);
      page.dispatchEvent(new Event(type));
      nextFrame?.();
      expect(freshFeedback.finish).not.toHaveBeenCalled();
    },
  );

  it("does not interfere with transitions during ordinary visible use", () => {
    const hover = new Transition("background-color");
    animations.push(hover);
    page.dispatchEvent(new Event("pointermove"));
    expect(hover.finish).not.toHaveBeenCalled();
    expect(nextFrame).toBeUndefined();
  });
});
