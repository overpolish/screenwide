// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { glideGesture } from "./glide-gesture-harness";

describe("Glide opening intent", () => {
  it.each([-1, 1])("accepts staggered diagonals on side %i", (x) => {
    for (const y of [-1, 1]) {
      for (const verticalFirst of [false, true]) {
        const gesture = glideGesture({ openingGraceMs: 30 });
        expect(
          gesture.move({
            deltaX: x * (verticalFirst ? 30 : 50),
            deltaY: y * (verticalFirst ? 50 : 30),
          }).changed,
        ).toBe(false);
        gesture.advance(20);
        const corner = gesture.move({
          deltaX: x * (verticalFirst ? 14 : 20),
          deltaY: y * (verticalFirst ? 20 : 14),
        });
        expect(corner.region).toMatchObject({
          colStart: x > 0 ? 1 : 0,
          rowSpan: 1,
          rowStart: y > 0 ? 1 : 0,
        });
        expect(corner.pending).toBeNull();
      }
    }
  });

  it("commits on a late timer and reports readiness once", () => {
    const gesture = glideGesture({ openingGraceMs: 30 });
    gesture.move({ deltaX: 50 });
    const result = gesture.detector.settle(100);
    expect(result.changed).toBe(true);
    expect(result.becameReady).toBe(true);
    expect(gesture.label).toBe("right half");
    expect(gesture.detector.settle(101).becameReady).toBe(false);
  });

  it("flushes a short flick on release and respects a modifier changed during grace", () => {
    const gesture = glideGesture({ openingGraceMs: 30 });
    gesture.move({ deltaX: 50 });
    gesture.detector.setThirds(true);
    expect(gesture.detector.finishOpening(10).changed).toBe(true);
    expect(gesture.label).toBe("right third");
    expect(gesture.detector.finishOpening(11).changed).toBe(false);
  });
});

describe("Glide turn eligibility", () => {
  it("does not refine a vertical middle-third opening", () => {
    const gesture = glideGesture();
    gesture.move({ deltaY: -50, thirds: true });
    expect(gesture.move({ deltaY: 50, thirds: true }).changed).toBe(false);
    expect(gesture.label).toBe("middle third");
  });

  it("can refine two thirds after a horizontal step", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 50, thirds: true });
    gesture.flick({ deltaX: -50, thirds: true });
    expect(gesture.label).toBe("right two thirds");
    expect(gesture.move({ deltaY: -50, thirds: true }).changed).toBe(true);
    expect(gesture.detector.region).toMatchObject({ colSpan: 2, rowSpan: 1 });
  });

  it("clears refinement travel on readiness", () => {
    const gesture = glideGesture();
    gesture.move({ deltaX: 50 });
    gesture.move({ deltaY: -30 });
    gesture.rest();
    expect(gesture.move({ deltaY: -14 }).changed).toBe(false);
  });
});
