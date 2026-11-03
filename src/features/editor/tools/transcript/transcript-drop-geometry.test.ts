// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { dropBoundary, dropCaret } from "./transcript-drop-geometry";
import { UnitRect } from "./transcript-line-geometry";

const rect = (left: number, top: number): UnitRect => ({
  bottom: top + 16,
  left,
  right: left + 40,
  top,
});

// Two lines of words 40px wide with 10px between them: the first line's at
// x 0, 50, 100 and y 0; the second's at x 0, 50 and y 20.
const rects = [
  rect(0, 0),
  rect(50, 0),
  rect(100, 0),
  rect(0, 20),
  rect(50, 20),
];
const ltr = () => false;
const rtl = () => true;
const before = (x: number, y: number, isRtl = ltr) =>
  dropBoundary(rects, { x, y }, isRtl)?.before;

describe("dropBoundary", () => {
  it("takes the side of the nearest word's middle the pointer is on", () => {
    expect(before(55, 8)).toBe(1);
    expect(before(85, 8)).toBe(2);
    // In the space between two words, the nearer one decides.
    expect(before(44, 8)).toBe(1);
  });

  it("reaches the end of a line and of the transcript past the last word", () => {
    expect(dropBoundary(rects, { x: 300, y: 8 }, ltr)).toEqual({
      before: 3,
      beside: 2,
    });
    expect(before(300, 28)).toBe(5);
  });

  it("takes the line nearest the pointer above or below the words", () => {
    expect(before(5, -30)).toBe(0);
    expect(before(5, 90)).toBe(3);
  });

  it("mirrors the sides of a word that reads right to left", () => {
    expect(before(55, 8, rtl)).toBe(2);
    expect(before(85, 8, rtl)).toBe(1);
  });

  it("finds nothing without a box to drop by", () => {
    expect(dropBoundary([undefined], { x: 0, y: 0 }, ltr)).toBeNull();
  });
});

describe("dropCaret", () => {
  it("stands in the middle of the space between two words", () => {
    const caret = { bottom: 16, top: 0, x: 45 };
    expect(dropCaret(rects, { before: 1, beside: 1 }, ltr)).toEqual(caret);
    expect(dropCaret(rects, { before: 1, beside: 0 }, ltr)).toEqual(caret);
  });

  it("shows a boundary where the line wraps on the line it was found on", () => {
    expect(dropCaret(rects, { before: 3, beside: 2 }, ltr)).toEqual({
      bottom: 16,
      top: 0,
      x: 140,
    });
    expect(dropCaret(rects, { before: 3, beside: 3 }, ltr)).toEqual({
      bottom: 36,
      top: 20,
      x: 0,
    });
  });

  it("stands on the right of a right-to-left line's first word", () => {
    expect(dropCaret(rects, { before: 0, beside: 0 }, rtl)).toEqual({
      bottom: 16,
      top: 0,
      x: 40,
    });
  });
});
