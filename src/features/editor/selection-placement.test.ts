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
  cropHeight: 500,
  cropWidth: 1000,
  cropX: 500,
  cropY: 250,
  imageWidth: 1000,
  imageX: 500,
  imageY: 250,
});

describe("selection placement in output pixels", () => {
  it("reads the crop as a top left rectangle", () => {
    expect(selectionPlacement(output())).toStrictEqual({
      height: 500,
      width: 1000,
      x: 500,
      y: 250,
    });
  });

  it("moves crop and image together, leaving the size alone", () => {
    const moved = withSelectionPlacement(output(), source, { x: 0, y: 0 });

    expect(selectionPlacement(moved)).toStrictEqual({
      height: 500,
      width: 1000,
      x: 0,
      y: 0,
    });
    // The image sits centred on the crop it was centred on before the move.
    expect(moved.imageX).toBeCloseTo(0);
    expect(moved.imageY).toBeCloseTo(0);
    expect(moved.imageWidth).toBeCloseTo(1000);
  });

  it("scales about the top left corner from whichever field was edited", () => {
    const scaled = withSelectionPlacement(output(), source, { width: 500 });

    expect(selectionPlacement(scaled)).toStrictEqual({
      height: 250,
      width: 500,
      x: 500,
      y: 250,
    });
    expect(scaled.imageWidth).toBeCloseTo(500);
  });

  it("keeps the placement when a field offers nothing usable", () => {
    const unchanged = withSelectionPlacement(output(), source, { width: 0 });

    expect(selectionPlacement(unchanged)).toStrictEqual(
      selectionPlacement(output()),
    );
  });
});
