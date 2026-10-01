// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { glideGesture } from "./glide-gesture-harness";

describe("GlideDetector fill", () => {
  it("steps from fill to full-width rows", () => {
    const gesture = glideGesture();
    gesture.move({ deltaY: -45 });

    expect(gesture.flick({ deltaY: -44 })).toBe("top half");
    expect(gesture.flick({ deltaY: 12 })).toBe("top half");
    expect(gesture.glide({ deltaY: 12 })).toBe("full screen");
    expect(gesture.flick({ deltaY: 50 })).toBe("bottom half");
  });

  it("takes a half out of fill without a full-width intermediate", () => {
    const gesture = glideGesture();
    gesture.move({ deltaY: -45 });

    expect(gesture.flick({ deltaX: 44 })).toBe("right half");
  });

  it("keeps the rows when fill narrows to a half", () => {
    const gesture = glideGesture();
    gesture.move({ deltaY: -45 });
    gesture.flick({ deltaY: -44 });

    expect(gesture.flick({ deltaX: -44 })).toBe("left half, top half");
  });
});

describe("GlideDetector thirds", () => {
  it("walks the ladder from the right third to the left third", () => {
    const gesture = glideGesture();
    const pull = { deltaX: -50, thirds: true };
    expect(gesture.glide({ deltaX: 45, thirds: true })).toBe("right third");

    expect(gesture.flick(pull)).toBe("right two thirds");
    expect(gesture.flick(pull)).toBe("middle third");
    expect(gesture.flick(pull)).toBe("left two thirds");
    expect(gesture.flick(pull)).toBe("left third");
    // The end of the ladder holds: there is nowhere further left to go.
    expect(gesture.flick(pull)).toBe("left third");
  });

  it.each([
    [-45, -50],
    [45, 50],
  ])("reaches the middle third two steps in from %i", (fold, step) => {
    const gesture = glideGesture();
    gesture.move({ deltaX: fold, thirds: true });

    gesture.flick({ deltaX: -step, thirds: true });
    expect(gesture.flick({ deltaX: -step, thirds: true })).toBe("middle third");
  });

  it("preserves the rows while a third grows", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45, thirds: true });
    const folded = gesture.flick({ deltaY: -44, thirds: true });
    expect(folded).toBe("right third, top half");

    const grown = gesture.flick({ deltaX: -50, thirds: true });
    expect(grown).toBe("right two thirds, top half");
  });

  it.each([
    [-44, "left two thirds"],
    [44, "right two thirds"],
  ])("steps the middle third by %i into the %s", (deltaX, expected) => {
    const gesture = glideGesture();
    gesture.move({ deltaY: -45, thirds: true });

    expect(gesture.flick({ deltaX, thirds: true })).toBe(expected);
  });

  it.each([
    [-44, "left two thirds"],
    [44, "right two thirds"],
  ])("joins the ladder from full width by %i at the %s", (deltaX, expected) => {
    const gesture = glideGesture();
    gesture.move({ deltaY: -45 });
    expect(gesture.detector.setThirds(true).region).not.toBeNull();
    expect(gesture.label).toBe("full screen");

    expect(gesture.flick({ deltaX, thirds: true })).toBe(expected);
  });
});

describe("GlideDetector re-gridding", () => {
  it("only changes the coming fold while no region exists", () => {
    const gesture = glideGesture();

    expect(gesture.detector.setThirds(true).changed).toBe(false);
    expect(gesture.glide({ deltaX: 45, thirds: true })).toBe("right third");
  });

  it.each([
    [-45, "left two thirds"],
    [45, "right two thirds"],
  ])("grows the half folded by %i into the %s", (deltaX, expected) => {
    const gesture = glideGesture();
    gesture.move({ deltaX });

    expect(gesture.detector.setThirds(true).changed).toBe(true);
    expect(gesture.label).toBe(expected);
  });

  it("keeps a third's own side and rows when it becomes a half", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 45, thirds: true });
    gesture.flick({ deltaY: -44, thirds: true });

    gesture.detector.setThirds(false);
    expect(gesture.label).toBe("right half, top half");
  });
});
