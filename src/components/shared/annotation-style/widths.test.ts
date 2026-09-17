// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  ANNOTATION_COUNTER_SIZES,
  ANNOTATION_WIDTHS,
  annotationSizes,
  annotationWidthAt,
  annotationWidthIndex,
  DEFAULT_ANNOTATION_COUNTER_SIZE,
  DEFAULT_ANNOTATION_WIDTH,
} from "./widths";

describe("the width presets", () => {
  it("maps every preset to its own place and back", () => {
    for (const [index, width] of ANNOTATION_WIDTHS.entries()) {
      expect(annotationWidthIndex(width)).toBe(index);
      expect(annotationWidthAt(index)).toBe(width);
    }
  });

  it("lands a width from an older document on the nearest preset", () => {
    expect(annotationWidthIndex(2)).toBe(0);
    expect(annotationWidthIndex(13)).toBe(ANNOTATION_WIDTHS.indexOf(12));
    expect(annotationWidthIndex(100)).toBe(ANNOTATION_WIDTHS.length - 1);
  });

  it("keeps the tool's own default when the width is not a number", () => {
    expect(annotationWidthAt(annotationWidthIndex(Number.NaN))).toBe(
      DEFAULT_ANNOTATION_WIDTH,
    );
  });

  it("clamps an index the control could never report", () => {
    expect(annotationWidthAt(-3)).toBe(ANNOTATION_WIDTHS[0]);
    expect(annotationWidthAt(99)).toBe(
      ANNOTATION_WIDTHS[ANNOTATION_WIDTHS.length - 1],
    );
  });
});

describe("the counter's own sizes", () => {
  it("runs the control over the disc sizes rather than the strokes", () => {
    const sizes = annotationSizes("counter");
    expect(sizes).toEqual(ANNOTATION_COUNTER_SIZES);
    expect(annotationSizes("arrow")).toEqual(ANNOTATION_WIDTHS);
    for (const [index, size] of sizes.entries()) {
      expect(annotationWidthIndex(size, sizes)).toBe(index);
      expect(annotationWidthAt(index, sizes)).toBe(size);
    }
  });

  it("is far enough apart to be worth choosing between", () => {
    // A disc a handful of pixels wider than the last is no choice at all, so
    // each size is at least half again the one before it.
    for (const [index, size] of ANNOTATION_COUNTER_SIZES.slice(1).entries())
      expect(size).toBeGreaterThanOrEqual(
        ANNOTATION_COUNTER_SIZES[index] * 1.5,
      );
  });

  it("starts a fresh counter at the smallest disc", () => {
    expect(ANNOTATION_COUNTER_SIZES[0]).toBe(DEFAULT_ANNOTATION_COUNTER_SIZE);
  });
});
