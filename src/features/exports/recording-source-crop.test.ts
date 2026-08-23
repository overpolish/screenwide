// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  applyScreenshotCropGesture,
  commitScreenshotCrop,
  uncroppedScreenshotPreviewOutput,
} from "./screenshot-crop";
import { sourceRect } from "./screenshot-geometry";
import { screenshotLayout } from "./screenshot-layout";
import { defaultScreenshotOutput } from "./screenshot-output-settings";
import { recenterScreenshotContent } from "./screenshot-recenter";

const source = { height: 1_000, width: 1_000 };

describe("recording Crop and Recenter parity", () => {
  it("uses the canonical screenshot Crop transaction", () => {
    const settings = {
      ...defaultScreenshotOutput(1_000, 1_000),
      recenterInsetColor: "#ffffff",
      screenshotCropHeightPercent: 80,
      screenshotCropWidthPercent: 80,
      screenshotCropXPercent: 10,
      screenshotCropYPercent: 10,
    };
    const live = applyScreenshotCropGesture({
      deltaX: 0.1,
      deltaY: 0,
      edges: 1,
      operation: "cropResize",
      output: source,
      settings,
      source,
    });
    const committed = commitScreenshotCrop(settings, live, source);
    const layout = screenshotLayout(source, source, committed);

    expect(committed.sourceCrop).toEqual(
      sourceRect({ height: 1, width: 0.9, x: 0.1, y: 0 }),
    );
    expect(layout.crop).toEqual({ height: 800, width: 700, x: 200, y: 100 });
    expect(
      uncroppedScreenshotPreviewOutput(source, committed).sourceCrop,
    ).toEqual(sourceRect({ height: 1, width: 1, x: 0, y: 0 }));
  });

  it("does not let Recenter recover source excluded by manual Crop", () => {
    const settings = defaultScreenshotOutput(1_000, 1_000);
    const live = applyScreenshotCropGesture({
      deltaX: 0.2,
      deltaY: 0,
      edges: 1,
      operation: "cropResize",
      output: source,
      settings,
      source,
    });
    const cropped = commitScreenshotCrop(settings, live, source);
    const recentered = recenterScreenshotContent(
      { ...cropped, recenterInsetColor: "#ffffff" },
      source,
      { height: 600, width: 600, x: 250, y: 200 },
    );

    expect(recentered.sourceCrop.x).toBeGreaterThanOrEqual(
      cropped.sourceCrop.x,
    );
    expect(
      recentered.sourceCrop.x + recentered.sourceCrop.width,
    ).toBeLessThanOrEqual(cropped.sourceCrop.x + cropped.sourceCrop.width);
  });
});
