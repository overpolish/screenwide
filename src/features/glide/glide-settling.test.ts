// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { glideGesture } from "./glide-gesture-harness";

describe("GlideDetector one gesture, one transition", () => {
  it("arms the minimize on a long downward flick without reaching the row", () => {
    const gesture = glideGesture();
    const detection = gesture.move({ deltaY: 300 });

    expect(detection.pending).toBe("minimize");
    expect(detection.region).toBeNull();
    expect(detection.phase).toBe("settling");
  });

  it("folds one step on a long sideways flick", () => {
    const gesture = glideGesture();

    // A distance-quantized ladder would walk several rungs per flick; each of
    // these pays for one, however far it travels.
    expect(gesture.glide({ deltaX: 400, thirds: true })).toBe("right third");
    expect(gesture.flick({ deltaX: -400, thirds: true })).toBe(
      "right two thirds",
    );
  });

  it("starts ready and returns to ready on reset", () => {
    const gesture = glideGesture();
    expect(gesture.detector.phase).toBe("ready");

    gesture.move({ deltaY: 45 });
    expect(gesture.detector.phase).toBe("settling");

    expect(gesture.detector.reset()).toEqual({
      becameReady: false,
      changed: true,
      pending: null,
      phase: "ready",
      region: null,
    });
  });
});

describe("GlideDetector rest gating", () => {
  it("discards motion that arrives before the fingers have rested", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });

    gesture.advance(40);
    // Sideways travel is discarded during every settle, porous or not; the
    // vertical axis after an opening fold is glide-corners' business.
    expect(gesture.move({ deltaX: -80 }).changed).toBe(false);
    expect(gesture.label).toBe("right half");
  });

  it("pushes the rest back for as long as the fingers keep moving", () => {
    // Pinned so the sub-rest gaps below stay meaningful whatever the default.
    const gesture = glideGesture({ restMs: 120 });
    gesture.move({ deltaY: 45 });

    for (let index = 0; index < 4; index += 1) {
      gesture.advance(100);
      expect(gesture.move({ deltaY: 60 }).phase).toBe("settling");
    }
    expect(gesture.detector.pending).toBe("minimize");

    // Only once the motion stops does the same flick count again.
    gesture.advance(120);
    const detection = gesture.move({ deltaY: 60 });
    expect(detection.becameReady).toBe(true);
    expect(detection.region).toEqual({
      colSpan: 2,
      colStart: 0,
      gridCols: 2,
      rowSpan: 1,
      rowStart: 1,
    });
  });

  it("counts jitter below the noise floor as rest", () => {
    const gesture = glideGesture();
    gesture.move({ deltaY: 45 });

    gesture.advance(40);
    expect(gesture.move({ deltaX: 1 }).phase).toBe("settling");
    gesture.advance(25);

    expect(gesture.settle()).toBe(true);
  });

  it("reports the rest still owed, and nothing once ready", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });

    expect(gesture.detector.restRemaining(50)).toBe(10);
    expect(gesture.rest()).toBe(true);
    expect(gesture.detector.restRemaining(200)).toBe(0);
  });
});

describe("GlideDetector became-ready", () => {
  it("does nothing when settle lands inside the rest", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });

    gesture.advance(59);
    expect(gesture.settle()).toBe(false);
    expect(gesture.detector.phase).toBe("settling");

    gesture.advance(1);
    expect(gesture.settle()).toBe(true);
    expect(gesture.detector.phase).toBe("ready");
  });

  it("reports the readiness once when settle wins the race", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });
    gesture.advance(130);

    expect(gesture.settle()).toBe(true);
    expect(gesture.settle()).toBe(false);
    // The event that follows the tick transitions, but does not tick again.
    expect(gesture.move({ deltaX: -50 }).becameReady).toBe(false);
    expect(gesture.label).toBe("left half");
  });

  it("reports the readiness once when the next sample wins the race", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });
    gesture.advance(130);

    const detection = gesture.move({ deltaX: -50 });
    expect(detection.becameReady).toBe(true);
    expect(detection.region).not.toBeNull();
    // The timer firing afterwards finds the rest already spent.
    expect(gesture.settle()).toBe(false);
  });

  it("never reports readiness from a re-grid", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });
    gesture.advance(130);

    expect(gesture.detector.setThirds(true).becameReady).toBe(false);
    expect(gesture.detector.phase).toBe("settling");
  });
});
