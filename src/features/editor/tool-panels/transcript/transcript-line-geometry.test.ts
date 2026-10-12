// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { nearestVerticalUnit, UnitRect } from "./transcript-line-geometry";

const rect = (left: number, top: number): UnitRect => ({
  bottom: top + 16,
  left,
  right: left + 40,
  top,
});

// Two lines of words, each 40px wide with no gap: the first line's words
// at x 0, 40, 80 and y 0; the second's at x 0, 60, 120 and y 20.
const rects = [
  rect(0, 0),
  rect(40, 0),
  rect(80, 0),
  rect(0, 20),
  rect(60, 20),
  rect(120, 20),
];

describe("nearestVerticalUnit", () => {
  it("moves down to the word nearest the same horizontal position", () => {
    expect(nearestVerticalUnit(rects, 1, 1)).toBe(4);
    expect(nearestVerticalUnit(rects, 2, 1)).toBe(4);
  });

  it("moves up to the word nearest the same horizontal position", () => {
    expect(nearestVerticalUnit(rects, 5, -1)).toBe(2);
    expect(nearestVerticalUnit(rects, 3, -1)).toBe(0);
  });

  it("stays put where there is no line in that direction", () => {
    expect(nearestVerticalUnit(rects, 0, -1)).toBe(0);
    expect(nearestVerticalUnit(rects, 5, 1)).toBe(5);
  });

  it("stays put where the unit itself has no box", () => {
    expect(nearestVerticalUnit(rects, 9, 1)).toBe(9);
  });

  it("treats tops within tolerance as the same line", () => {
    const wobbly = [rect(0, 0), rect(0, 2)];
    expect(nearestVerticalUnit(wobbly, 0, 1)).toBe(0);
  });
});
