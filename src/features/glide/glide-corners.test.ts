// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { glideGesture, type GlideStroke } from "./glide-gesture-harness";

describe("GlideDetector diagonal first fold", () => {
  const diagonals: [GlideStroke, string][] = [
    [{ deltaX: 45, deltaY: -45 }, "right half, top half"],
    [{ deltaX: -45, deltaY: -45 }, "left half, top half"],
    [{ deltaX: 45, deltaY: 45 }, "right half, bottom half"],
    [{ deltaX: -45, deltaY: 45 }, "left half, bottom half"],
    [{ deltaX: 45, deltaY: -45, thirds: true }, "right third, top half"],
    [{ deltaX: -45, deltaY: 45, thirds: true }, "left third, bottom half"],
  ];

  it.each(diagonals)(
    "folds the %o diagonal straight into the %s",
    (stroke, expected) => {
      const gesture = glideGesture();

      expect(gesture.glide(stroke)).toBe(expected);
      // The corner is a move, never an armed action, whichever way it points.
      expect(gesture.detector.pending).toBeNull();
    },
  );

  it.each([
    // Just outside the cone: 44 is under half of 100, so the flick is straight.
    [100, -44, "right half"],
    [100, -60, "right half, top half"],
    [45, -45, "right half, top half"],
    // Vertical-leaning but inside the cone: a corner, not an armed minimize.
    [45, 60, "right half, bottom half"],
  ])("lands %i/%i on the %s", (deltaX, deltaY, expected) => {
    const gesture = glideGesture();

    expect(gesture.glide({ deltaX, deltaY })).toBe(expected);
    expect(gesture.detector.pending).toBeNull();
  });

  it("spends the whole window: the corner fold settles hard", () => {
    const gesture = glideGesture();
    expect(gesture.glide({ deltaX: 45, deltaY: -45 })).toBe(
      "right half, top half",
    );

    // The diagonal fold is not the opening sideways fold, so nothing is porous.
    expect(gesture.move({ deltaY: -60 }).changed).toBe(false);
    expect(gesture.move({ deltaX: -60 }).changed).toBe(false);
    expect(gesture.label).toBe("right half, top half");
  });
});

describe("GlideDetector porous opening settle", () => {
  /** The opening sideways fold, whose settle stays open to the rows. */
  const openSideways = (thirds = false) => {
    const gesture = glideGesture();
    expect(gesture.glide({ deltaX: 50, thirds })).toBe(
      thirds ? "right third" : "right half",
    );
    return gesture;
  };

  it.each([
    [false, -50, "right half, top half"],
    [false, 50, "right half, bottom half"],
    [true, -50, "right third, top half"],
  ])(
    "turns an L-curve (thirds %s, %i) into the %s without stopping",
    (thirds, deltaY, expected) => {
      const gesture = openSideways(thirds);

      const converted = gesture.move({ deltaY, thirds });
      expect(converted.changed).toBe(true);
      expect(converted.region).not.toBeNull();
      expect(gesture.label).toBe(expected);
      // Down-right is the bottom corner, never the armed minimize.
      expect(converted.pending).toBeNull();
    },
  );

  it("closes the porosity behind the conversion", () => {
    const gesture = openSideways();
    gesture.move({ deltaY: -50 });

    // Whatever the same motion had left buys nothing more, either axis.
    expect(gesture.move({ deltaY: -50 }).changed).toBe(false);
    expect(gesture.move({ deltaX: 60 }).changed).toBe(false);
    expect(gesture.label).toBe("right half, top half");
  });

  it("keeps discarding horizontal travel while porous", () => {
    const gesture = openSideways();

    expect(gesture.move({ deltaX: -80 }).changed).toBe(false);
    expect(gesture.label).toBe("right half");
    // Discarded, but the vertical axis is still listening.
    expect(gesture.move({ deltaY: -50 }).changed).toBe(true);
  });

  it("stirs the rest on either axis while porous", () => {
    const gesture = glideGesture({ restMs: 120 });
    gesture.move({ deltaX: 50 });

    gesture.advance(100);
    expect(gesture.move({ deltaX: -80 }).phase).toBe("settling");
    gesture.advance(100);
    // The horizontal travel pushed the rest back, so this is still one motion.
    expect(gesture.move({ deltaY: -50 }).changed).toBe(true);
  });

  it("re-holds the rest, so a single tick follows the corner", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 50 });

    gesture.advance(50);
    // A settling transition announces no readiness of its own.
    expect(gesture.move({ deltaY: -50 }).becameReady).toBe(false);
    // The rest restarts at the conversion: the first 50ms bought nothing.
    gesture.advance(50);
    expect(gesture.settle()).toBe(false);

    gesture.advance(10);
    expect(gesture.settle()).toBe(true);
    expect(gesture.settle()).toBe(false);
    expect(gesture.label).toBe("right half, top half");
  });

  it("closes the porosity when the rest completes instead", () => {
    const gesture = openSideways();
    gesture.rest();

    // The paced flick-rest-flick corner is untouched: the row costs its own
    // full travel from the ready window, not the settling one.
    expect(gesture.move({ deltaY: -20 }).changed).toBe(false);
    expect(gesture.glide({ deltaY: -30 })).toBe("right half, top half");
  });

  it("closes the porosity on reset", () => {
    const gesture = openSideways();
    gesture.detector.reset();

    // Reset returns a fresh gesture: the vertical flick opens its own fold.
    expect(gesture.glide({ deltaY: -50 })).toBe("full screen");
  });

  it("never arms porosity on a vertical first fold", () => {
    const gesture = glideGesture();
    expect(gesture.glide({ deltaY: -45 })).toBe("full screen");

    expect(gesture.move({ deltaX: 80 }).changed).toBe(false);
    expect(gesture.label).toBe("full screen");
  });

  it("never arms porosity on a later horizontal fold", () => {
    const gesture = openSideways();
    gesture.rest();

    // A ladder step is not an opening fold, however sideways it is.
    expect(gesture.move({ deltaX: -60 }).changed).toBe(true);
    expect(gesture.move({ deltaY: -50 }).changed).toBe(false);
    expect(gesture.label).toBe("left half");
  });

  it("never arms porosity on the sideways escape from the arm", () => {
    const gesture = glideGesture();
    gesture.move({ deltaY: 45 });
    gesture.rest();

    expect(gesture.move({ deltaX: 60 }).pending).toBeNull();
    expect(gesture.move({ deltaY: -50 }).changed).toBe(false);
    expect(gesture.label).toBe("right half");
  });
});
