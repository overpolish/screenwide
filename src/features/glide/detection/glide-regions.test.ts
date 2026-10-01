// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  describeRegion,
  type GlideRegion,
  regridRegion,
  stepColumns,
  stepRows,
} from "./glide-regions";

/** A full-height left cell, refined per case. */
const cell = (
  cells: Partial<GlideRegion> & Pick<GlideRegion, "gridCols">,
): GlideRegion => ({
  colSpan: 1,
  colStart: 0,
  rowSpan: 2,
  rowStart: 0,
  ...cells,
});

const half = (cells: Partial<GlideRegion> = {}) =>
  cell({ gridCols: 2, ...cells });

const third = (cells: Partial<GlideRegion> = {}) =>
  cell({ gridCols: 3, ...cells });

/** Names where a run of same-direction column steps lands, rung by rung. */
const walkColumns = (from: GlideRegion, step: number, steps: number) => {
  const names: string[] = [];
  let region = from;
  for (let index = 0; index < steps; index += 1) {
    region = stepColumns(region, step);
    names.push(describeRegion(region));
  }
  return names;
};

describe("describeRegion", () => {
  it.each([
    [half({ colSpan: 2 }), "full screen"],
    [half(), "left half"],
    [half({ colSpan: 2, rowSpan: 1, rowStart: 1 }), "bottom half"],
    [half({ colStart: 1, rowSpan: 1 }), "right half, top half"],
    [third({ colStart: 1 }), "middle third"],
    [
      third({ colSpan: 2, colStart: 1, rowSpan: 1 }),
      "right two thirds, top half",
    ],
  ])("names a region as %#: %s", (region, expected) => {
    expect(describeRegion(region)).toBe(expected);
  });
});

describe("stepColumns on thirds", () => {
  it("walks the ladder down from the right third to the left third", () => {
    expect(walkColumns(third({ colStart: 2 }), -1, 5)).toEqual([
      "right two thirds",
      "middle third",
      "left two thirds",
      "left third",
      // The end of the ladder holds rather than wrapping or widening.
      "left third",
    ]);
  });

  it("walks the same ladder back up to the right third", () => {
    expect(walkColumns(third(), 1, 5)).toEqual([
      "left two thirds",
      "middle third",
      "right two thirds",
      "right third",
      "right third",
    ]);
  });

  it.each([
    [third({ colStart: 2 }), -1],
    [third(), 1],
  ])("reaches the middle third in two steps from %#", (from, step) => {
    const once = stepColumns(from, step);

    expect(describeRegion(stepColumns(once, step))).toBe("middle third");
  });

  it.each([
    [-1, "left two thirds"],
    [1, "right two thirds"],
  ])("joins the ladder from full width by %i at the %s", (step, expected) => {
    // Full width is off the ladder: only a re-gridded fill starts there.
    expect(describeRegion(stepColumns(third({ colSpan: 3 }), step))).toBe(
      expected,
    );
  });

  it("preserves the rows a third already has", () => {
    const stepped = stepColumns(third({ colStart: 2, rowSpan: 1 }), -1);

    expect(describeRegion(stepped)).toBe("right two thirds, top half");
  });
});

describe("stepColumns on halves", () => {
  it("narrows full width to the side of the step", () => {
    expect(describeRegion(stepColumns(half({ colSpan: 2 }), 1))).toBe(
      "right half",
    );
  });

  it("holds a half pushed into its own edge", () => {
    const region = half({ colStart: 1, rowSpan: 1 });

    expect(stepColumns(region, 1)).toBe(region);
  });

  it("toggles a full-height half straight across", () => {
    expect(describeRegion(stepColumns(half({ colStart: 1 }), -1))).toBe(
      "left half",
    );
  });

  it("carries a row to the far corner through the full-width row", () => {
    expect(walkColumns(half({ rowSpan: 1 }), 1, 3)).toEqual([
      "top half",
      "right half, top half",
      "right half, top half",
    ]);
  });
});

describe("stepRows", () => {
  it.each([
    [half(), 1, "left half, bottom half"],
    [half(), -1, "left half, top half"],
    [half({ rowSpan: 1 }), 1, "left half"],
    [half({ rowSpan: 1, rowStart: 1 }), -1, "left half"],
  ])("steps rows of %# by %i into the %s", (region, step, expected) => {
    expect(describeRegion(stepRows(region, step))).toBe(expected);
  });

  it("holds a row pushed into its own edge", () => {
    const region = half({ rowSpan: 1 });

    expect(stepRows(region, -1)).toBe(region);
  });
});

describe("regridRegion", () => {
  it.each([
    [half(), "left two thirds"],
    [half({ colStart: 1 }), "right two thirds"],
    [half({ colSpan: 2 }), "full screen"],
  ])("widens %# onto the thirds grid as the %s", (region, expected) => {
    expect(describeRegion(regridRegion(region, true))).toBe(expected);
  });

  it.each([
    [third(), "left half"],
    [third({ colStart: 2 }), "right half"],
    [third({ colSpan: 2 }), "left half"],
    [third({ colSpan: 2, colStart: 1 }), "right half"],
    // Neither full width nor the middle third has a half to anchor to.
    [third({ colSpan: 3 }), "full screen"],
    [third({ colStart: 1 }), "full screen"],
  ])("anchors %# onto the halves grid as the %s", (region, expected) => {
    expect(describeRegion(regridRegion(region, false))).toBe(expected);
  });

  it("keeps the rows through a re-grid either way", () => {
    const narrowed = regridRegion(
      third({ colStart: 2, rowSpan: 1, rowStart: 1 }),
      false,
    );
    const widened = regridRegion(half({ colStart: 1, rowSpan: 1 }), true);

    expect(describeRegion(narrowed)).toBe("right half, bottom half");
    expect(describeRegion(widened)).toBe("right two thirds, top half");
  });
});
