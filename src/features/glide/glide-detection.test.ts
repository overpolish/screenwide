// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { glideGesture } from "./glide-gesture-harness";

describe("GlideDetector first fold", () => {
  it("ignores movement below the first-fold threshold", () => {
    const gesture = glideGesture();

    expect(gesture.move({ deltaX: 30, deltaY: 2 }).region).toBeNull();
  });

  it.each([
    [-45, false, "left half"],
    [45, false, "right half"],
    [-45, true, "left third"],
    [45, true, "right third"],
  ])("folds %i (thirds %s) into the %s", (deltaX, thirds, expected) => {
    const gesture = glideGesture();

    expect(gesture.glide({ deltaX, thirds })).toBe(expected);
  });

  it("rejects a first fold dominated by vertical movement", () => {
    const gesture = glideGesture();
    // Too far off the corner cone to be a diagonal, so the downward half of
    // the flick owns it instead - see glide-corners and glide-minimize.
    expect(gesture.glide({ deltaX: 45, deltaY: 100 })).toBeNull();
    expect(gesture.detector.pending).toBe("minimize");
  });

  it.each([
    [false, "full screen"],
    [true, "middle third"],
  ])("folds straight up (thirds %s) into the %s", (thirds, expected) => {
    const gesture = glideGesture();

    expect(gesture.glide({ deltaY: -45, thirds })).toBe(expected);
  });

  it("requires the fill fold to dominate a diagonal approach", () => {
    const gesture = glideGesture();

    // Short of the corner cone's horizontal threshold and short of dominance:
    // no transition, so the approach is still the same gesture, and
    // straightening out lets the upward travel take the fold after all.
    expect(gesture.glide({ deltaX: -42, deltaY: -46 })).toBeNull();
    expect(gesture.glide({ deltaY: -20 })).toBe("full screen");
  });

  it("does not count pre-fold vertical movement toward a row", () => {
    const gesture = glideGesture();
    gesture.move({ deltaY: -30 });

    expect(gesture.glide({ deltaX: -60 })).toBe("left half");
    expect(gesture.flick({ deltaY: -20 })).toBe("left half");
    expect(gesture.glide({ deltaY: -30 })).toBe("left half, top half");
  });
});

describe("GlideDetector halves", () => {
  it("switches sides after a decisive horizontal reversal", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: -45 });

    expect(gesture.flick({ deltaX: 50 })).toBe("right half");
  });

  it("measures a reversal from the turn point, not the window's start", () => {
    const gesture = glideGesture();
    expect(gesture.glide({ deltaX: 45 })).toBe("right half");
    gesture.rest();

    // 30px out and 54px back is a 24px net that would never buy a step; the
    // turn point re-origins at the extremum, so the swipe back measures 54.
    gesture.move({ deltaX: 30 });
    expect(gesture.glide({ deltaX: -18 })).toBe("right half");
    expect(gesture.glide({ deltaX: -18 })).toBe("right half");
    expect(gesture.glide({ deltaX: -18 })).toBe("left half");
  });

  it("reaches the rows from a corner without restarting", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: -45 });
    expect(gesture.flick({ deltaY: -50 })).toBe("left half, top half");

    // The row rides along: the step grows across before shrinking again.
    expect(gesture.flick({ deltaX: 50 })).toBe("top half");
    expect(gesture.flick({ deltaX: 50 })).toBe("right half, top half");
  });

  it("climbs the row ladder with the release/step asymmetry", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });

    expect(gesture.flick({ deltaY: -25 })).toBe("right half");
    expect(gesture.glide({ deltaY: -25 })).toBe("right half, top half");

    // Coming back costs verticalReleaseThreshold (20) to reach full height,
    // then verticalThreshold (44) to reach the opposite row.
    expect(gesture.flick({ deltaY: 12 })).toBe("right half, top half");
    expect(gesture.glide({ deltaY: 12 })).toBe("right half");
    expect(gesture.flick({ deltaY: 20 })).toBe("right half");
    expect(gesture.glide({ deltaY: 30 })).toBe("right half, bottom half");
  });

  it("holds its region through jitter below the reversal hysteresis", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });
    gesture.flick({ deltaY: -50 });
    gesture.rest();

    for (let index = 0; index < 4; index += 1) {
      expect(gesture.move({ deltaX: -8, deltaY: 8 }).changed).toBe(false);
      expect(gesture.move({ deltaX: 8, deltaY: -8 }).changed).toBe(false);
    }

    expect(gesture.label).toBe("right half, top half");
  });

  it("reports only actual region transitions as changes", () => {
    const gesture = glideGesture();

    expect(gesture.move({ deltaX: -45 }).changed).toBe(true);
    gesture.rest();
    expect(gesture.move({ deltaX: -5 }).changed).toBe(false);
    expect(gesture.move({ deltaY: -45 }).changed).toBe(true);
  });
});

describe("GlideDetector deliberate corners", () => {
  /** Folds right, then slides all the way left with a steady upward drift. */
  const slideLeft = () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45 });
    for (let index = 0; index < 6; index += 1) {
      gesture.flick({ deltaX: -50, deltaY: -15 });
    }
    return gesture;
  };

  it("never brushes into a row during a long sideways slide", () => {
    // 90px of accumulated drift would twice pay for a row; every horizontal
    // step wipes it, so the slide lands squarely on the half.
    expect(slideLeft().label).toBe("left half");
  });

  it("still takes a corner on a deliberate fold once the slide settles", () => {
    const gesture = slideLeft();

    expect(gesture.flick({ deltaY: -20 })).toBe("left half");
    expect(gesture.glide({ deltaY: -30 })).toBe("left half, top half");
  });

  it("releases that corner for the smaller release cost", () => {
    const gesture = slideLeft();
    gesture.flick({ deltaY: -50 });

    expect(gesture.flick({ deltaY: 12 })).toBe("left half, top half");
    expect(gesture.glide({ deltaY: 12 })).toBe("left half");
  });
});
