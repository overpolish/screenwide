// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "./screenshot-output";
import {
  selectionPlacement,
  withSelectionPlacement,
} from "./selection-placement";

const source = { height: 1000, width: 2000 };
const output = () => ({
  ...defaultScreenshotOutput(2000, 1000),
  screenshotCropHeightPercent: 50,
  screenshotCropWidthPercent: 50,
  screenshotCropXPercent: 25,
  screenshotCropYPercent: 25,
  screenshotImageWidthPercent: 50,
  screenshotImageXPercent: 50,
  screenshotImageYPercent: 50,
});

describe("selection placement in output pixels", () => {
  it("reads the crop as a top left rectangle", () => {
    expect(selectionPlacement(output(), source)).toStrictEqual({
      height: 500,
      width: 1000,
      x: 500,
      y: 250,
    });
  });

  it("moves crop and image together, leaving the size alone", () => {
    const moved = withSelectionPlacement(output(), source, { x: 0, y: 0 });

    expect(selectionPlacement(moved, source)).toStrictEqual({
      height: 500,
      width: 1000,
      x: 0,
      y: 0,
    });
    // The image sits centred on the crop it was centred on before the move.
    expect(moved.screenshotImageXPercent).toBeCloseTo(25);
    expect(moved.screenshotImageYPercent).toBeCloseTo(25);
    expect(moved.screenshotImageWidthPercent).toBeCloseTo(50);
  });

  it("scales about the top left corner from whichever field was edited", () => {
    const scaled = withSelectionPlacement(output(), source, { width: 500 });

    expect(selectionPlacement(scaled, source)).toStrictEqual({
      height: 250,
      width: 500,
      x: 500,
      y: 250,
    });
    expect(scaled.screenshotImageWidthPercent).toBeCloseTo(25);
  });

  it("keeps the placement when a field offers nothing usable", () => {
    const unchanged = withSelectionPlacement(output(), source, { width: 0 });

    expect(selectionPlacement(unchanged, source)).toStrictEqual(
      selectionPlacement(output(), source),
    );
  });
});
