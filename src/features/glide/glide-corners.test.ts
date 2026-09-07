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

  it.each(diagonals)("folds %o into %s", (stroke, expected) => {
    const gesture = glideGesture({ openingGraceMs: 30 });
    expect(gesture.glide(stroke)).toBe(expected);
    expect(gesture.detector.pending).toBeNull();
  });

  it.each([
    [100, -44, "right half"],
    [100, -60, "right half, top half"],
    [45, -45, "right half, top half"],
    [45, 60, "right half, bottom half"],
  ])("lands %i/%i on the %s", (deltaX, deltaY, expected) => {
    const gesture = glideGesture();
    expect(gesture.glide({ deltaX, deltaY })).toBe(expected);
    expect(gesture.detector.pending).toBeNull();
  });

  it("discards all axes while a diagonal settles", () => {
    const gesture = glideGesture();
    expect(gesture.glide({ deltaX: 45, deltaY: -45 })).toBe(
      "right half, top half",
    );
    expect(gesture.move({ deltaY: -60 }).changed).toBe(false);
    expect(gesture.move({ deltaX: -60 }).changed).toBe(false);
    expect(gesture.label).toBe("right half, top half");
  });
});

describe("GlideDetector settling", () => {
  it("allows one corner refinement then discards further movement", () => {
    const gesture = glideGesture();
    expect(gesture.glide({ deltaX: 50 })).toBe("right half");
    expect(gesture.move({ deltaY: -50 }).changed).toBe(true);
    expect(gesture.label).toBe("right half, top half");
    expect(gesture.move({ deltaX: -80, deltaY: 80 }).changed).toBe(false);
    expect(gesture.label).toBe("right half, top half");
    expect(gesture.flick({ deltaY: -50 })).toBe("right half, top half");
  });
});
