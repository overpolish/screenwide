// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { normalizedScreenshotSelection } from "./components/screenshot-selection";
import { sourceRect } from "./screenshot-geometry";
import { screenshotLayout } from "./screenshot-layout";
import {
  defaultScreenshotOutput,
  normalizedScreenshotOutput,
  screenshotSourceCrop,
  withScreenshotSourceCrop,
} from "./screenshot-output-settings";
import { recenterScreenshotContent } from "./screenshot-recenter";

const expectRectClose = (
  actual: { height: number; width: number; x: number; y: number } | null,
  expected: { height: number; width: number; x: number; y: number },
) => {
  expect(actual).not.toBeNull();
  expect(actual?.height).toBeCloseTo(expected.height);
  expect(actual?.width).toBeCloseTo(expected.width);
  expect(actual?.x).toBeCloseTo(expected.x);
  expect(actual?.y).toBeCloseTo(expected.y);
};

describe("persisted screenshot source crop", () => {
  it("normalizes and persists one canonical crop", () => {
    const crop = sourceRect({ height: 0.6, width: 0.7, x: 0.2, y: 0.1 });
    const output = normalizedScreenshotOutput(
      withScreenshotSourceCrop(defaultScreenshotOutput(1_000, 1_000), crop),
    );

    expect(output.sourceCrop).toEqual(crop);
  });

  it("lays out the canonical crop directly", () => {
    const canonical = withScreenshotSourceCrop(
      defaultScreenshotOutput(1_000, 1_000),
      sourceRect({ height: 0.5, width: 0.4, x: 0.2, y: 0.3 }),
    );
    const layout = screenshotLayout({ height: 1_000, width: 1_000 }, canonical);

    expectRectClose(layout.sourceCrop, {
      height: 500,
      width: 400,
      x: 200,
      y: 300,
    });
  });
});

describe("Crop and padding composition", () => {
  it("describes the padded frame as the layer the Select tool places", () => {
    const selection = normalizedScreenshotSelection(
      {
        crop: { height: 600, width: 600, x: 200, y: 200 },
        image: { height: 1_000, width: 1_000, x: 0, y: 0 },
        sourceCrop: { height: 400, width: 400, x: 300, y: 300 },
      },
      { height: 1_000, width: 1_000 },
      "select",
    );

    expect(selection.rect).toEqual({
      height: 0.6,
      width: 0.6,
      x: 0.2,
      y: 0.2,
    });
    expect(selection.image).toEqual({
      height: 1,
      width: 1,
      x: 0,
      y: 0,
    });
    expect(selection.recenterBounds).toBeUndefined();
  });

  it("writes the detected content into the layer's crop", () => {
    const cropped = withScreenshotSourceCrop(
      {
        ...defaultScreenshotOutput(1_000, 1_000),
        recenterInsetColor: "#ffffff",
      },
      sourceRect({ height: 0.6, width: 0.6, x: 0.2, y: 0.2 }),
    );
    const recentered = recenterScreenshotContent(
      cropped,
      { height: 1_000, width: 1_000 },
      { height: 500, width: 500, x: 100, y: 100 },
    );

    const detectedCrop = sourceRect({
      height: 0.5,
      width: 0.5,
      x: 0.1,
      y: 0.1,
    });
    expect(screenshotSourceCrop(recentered)).toEqual(detectedCrop);
    // The colour the pad is filled with survives: it belongs to the layer
    // rather than to any one inset.
    expect(recentered.recenterInsetColor).toBe("#ffffff");
  });

  it("preserves the configured inset when Recenter detects new content", () => {
    const source = { height: 1_000, width: 1_000 };
    const settings = withScreenshotSourceCrop(
      {
        ...defaultScreenshotOutput(source.width, source.height),
        cropHeight: 600,
        cropWidth: 600,
        cropX: 200,
        cropY: 200,
        recenterInsetColor: "#ffffff",
      },
      sourceRect({ height: 0.4, width: 0.4, x: 0.3, y: 0.3 }),
    );

    const recentered = recenterScreenshotContent(settings, source, {
      height: 200,
      width: 200,
      x: 400,
      y: 400,
    });
    const layout = screenshotLayout(source, recentered);

    expect((layout.crop.width - layout.sourceCrop.width) / 2).toBeCloseTo(100);
    expect((layout.crop.height - layout.sourceCrop.height) / 2).toBeCloseTo(
      100,
    );
  });
});
