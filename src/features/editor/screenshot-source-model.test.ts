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
import {
  applyScreenshotRecenterGesture,
  recenterScreenshotContent,
  resetScreenshotRecenter,
} from "./screenshot-recenter";

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

describe("Crop and Recenter composition", () => {
  it("describes Recenter as an outer frame around fixed visible content", () => {
    const selection = normalizedScreenshotSelection(
      {
        crop: { height: 600, width: 600, x: 200, y: 200 },
        image: { height: 1_000, width: 1_000, x: 0, y: 0 },
        sourceCrop: { height: 400, width: 400, x: 300, y: 300 },
      },
      { height: 1_000, width: 1_000 },
      "recenter",
    );

    expect(selection.rect).toEqual({
      height: 0.6,
      width: 0.6,
      x: 0.2,
      y: 0.2,
    });
    expect(selection.image).toEqual({
      height: 0.4,
      width: 0.4,
      x: 0.3,
      y: 0.3,
    });
    expect(selection.recenterBounds).toEqual({
      height: 1,
      width: 1,
      x: 0,
      y: 0,
    });
  });

  it("resizes the Recenter inset without changing the visible source", () => {
    const source = { height: 1_000, width: 1_000 };
    const settings = withScreenshotSourceCrop(
      {
        ...defaultScreenshotOutput(1_000, 1_000),
        cropHeight: 400,
        cropWidth: 400,
        cropX: 300,
        cropY: 300,
      },
      sourceRect({ height: 0.4, width: 0.4, x: 0.3, y: 0.3 }),
    );
    const before = screenshotLayout(source, settings);

    const resized = applyScreenshotRecenterGesture({
      deltaX: 0,
      deltaY: 0,
      edges: 2,
      operation: "resize",
      scale: 1.5,
      settings,
      source,
    });

    expect(resized).not.toBeNull();
    if (!resized) throw new Error("Expected Recenter resize settings");
    expect(resized.sourceCrop).toEqual(settings.sourceCrop);
    expect(resized.imageWidth).toBe(settings.imageWidth);
    expect(resized.imageX).toBe(settings.imageX);
    expect(resized.imageY).toBe(settings.imageY);
    const after = screenshotLayout(source, resized);
    expectRectClose(after.sourceCrop, before.sourceCrop);
    expectRectClose(after.crop, {
      height: 600,
      width: 600,
      x: 200,
      y: 200,
    });
  });

  it("clamps a growing Recenter inset uniformly to the canvas", () => {
    const source = { height: 1_000, width: 1_000 };
    const settings = withScreenshotSourceCrop(
      {
        ...defaultScreenshotOutput(1_000, 1_000),
        cropHeight: 800,
        cropWidth: 800,
        cropX: 100,
        cropY: 100,
      },
      sourceRect({ height: 0.8, width: 0.8, x: 0.1, y: 0.1 }),
    );

    const resized = applyScreenshotRecenterGesture({
      deltaX: 0,
      deltaY: 0,
      edges: 8,
      operation: "resize",
      scale: 10,
      settings,
      source,
    });

    expect(resized).not.toBeNull();
    if (!resized) throw new Error("Expected clamped Recenter settings");
    expectRectClose(screenshotLayout(source, resized).crop, {
      height: 1_000,
      width: 1_000,
      x: 0,
      y: 0,
    });
  });

  it("contracts the Recenter frame no farther than the fixed content", () => {
    const source = { height: 1_000, width: 1_000 };
    const settings = withScreenshotSourceCrop(
      {
        ...defaultScreenshotOutput(1_000, 1_000),
        cropHeight: 800,
        cropWidth: 800,
        cropX: 100,
        cropY: 100,
      },
      sourceRect({ height: 0.2, width: 0.2, x: 0.4, y: 0.4 }),
    );

    const resized = applyScreenshotRecenterGesture({
      deltaX: 0,
      deltaY: 0,
      edges: 1,
      operation: "resize",
      scale: 0,
      settings,
      source,
    });

    expect(resized).not.toBeNull();
    if (!resized) throw new Error("Expected contracted Recenter settings");
    const layout = screenshotLayout(source, resized);
    expectRectClose(layout.crop, layout.sourceCrop);
  });

  it("writes Recenter into Crop and removes only its inset on reset", () => {
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

    const inset = {
      ...recentered,
      cropHeight: 600,
      cropWidth: 600,
      cropX: 200,
      cropY: 200,
    };
    const reset = resetScreenshotRecenter(inset, {
      height: 1_000,
      width: 1_000,
    });
    expect(reset.sourceCrop).toEqual(detectedCrop);
    expect(reset.recenterInsetColor).toBeNull();
    expectRectClose(
      screenshotLayout({ height: 1_000, width: 1_000 }, reset).crop,
      screenshotLayout({ height: 1_000, width: 1_000 }, reset).sourceCrop,
    );
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
