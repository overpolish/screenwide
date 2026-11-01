// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { comparedPlacement } from "./export-compare-placement";

import type { ToolPanelComparedFrame } from "../../tool-panels/tool-panel-export";

/** A frame `width` by `height` showing the canvas at `canvas`. */
const frame = (
  width: number,
  height: number,
  canvas: ToolPanelComparedFrame["canvas"],
): ToolPanelComparedFrame => ({ canvas, height, src: "", width });

/** Where a row `y` of the exported frame lands on the Original frame. */
const shownRow = (
  y: number,
  original: ToolPanelComparedFrame,
  exported: ToolPanelComparedFrame,
) => {
  const at = comparedPlacement(original, exported);
  return at.top + (y * at.height) / exported.height;
};

describe("comparedPlacement", () => {
  // A 3600 by 2338 canvas at Smaller is 2700 by 1753.5, centred a quarter
  // pixel down in a 2700 by 1754 frame; at Smallest 1169 rows, half a
  // pixel cropped from a 1168 row frame.
  const original = frame(3_600, 2_338, {
    height: 2_338,
    width: 3_600,
    x: 0,
    y: 0,
  });
  const smaller = frame(2_700, 1_754, {
    height: 1_753.5,
    width: 2_700,
    x: 0,
    y: 0.25,
  });
  const smallest = frame(1_800, 1_168, {
    height: 1_169,
    width: 1_800,
    x: 0,
    y: -0.5,
  });

  it.each([
    { exported: smaller, look: "Smaller", scale: 0.75 },
    { exported: smallest, look: "Smallest", scale: 0.5 },
  ])("lines up the top and bottom rows at $look", ({ exported, scale }) => {
    for (const row of [0, 1, 1_169, 2_337, 2_338]) {
      // The row as the export draws it, carried back onto the Original.
      const drawn = exported.canvas.y + row * scale;
      const shown = shownRow(drawn, original, exported);
      expect(Math.abs(shown - row)).toBeLessThanOrEqual(0.5);
    }
  });

  it("draws the exported frame at one scale on both axes", () => {
    const at = comparedPlacement(original, smaller);
    expect(at.width / smaller.width).toBeCloseTo(
      at.height / smaller.height,
      12,
    );
    expect(at).toEqual({
      height: (1_754 * 2_338) / 1_753.5,
      left: 0,
      top: (-0.25 * 2_338) / 1_753.5,
      width: 3_600,
    });
  });

  it("stretches a frame FFmpeg filled back over the whole Original", () => {
    const filled = frame(2_700, 1_754, {
      height: 1_754,
      width: 2_700,
      x: 0,
      y: 0,
    });
    expect(comparedPlacement(original, filled)).toEqual({
      height: 2_338,
      left: 0,
      top: 0,
      width: 3_600,
    });
  });
});
