// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { commitScreenshotCrop } from "./screenshot-crop";
import { sourceRect } from "./screenshot-geometry";
import {
  INSET_LOOKS,
  insetScreenshotPadding,
  screenshotInsetShare,
} from "./screenshot-inset";
import {
  defaultScreenshotOutput,
  screenshotLayout,
  ScreenshotOutputSettings,
} from "./screenshot-output";
import { withScreenshotSourceCrop } from "./screenshot-output-settings";
import { recenterScreenshotContent } from "./screenshot-recenter";

const source = { height: 1_000, width: 1_000 };

/** The whole capture, 1000 pixels square, on a canvas of its size. */
const whole = () =>
  withScreenshotSourceCrop(
    defaultScreenshotOutput(source.width, source.height),
    sourceRect({ height: 1, width: 1, x: 0, y: 0 }),
  );

/** A layer saved while the frame alone carried its padding: `pixels` on
 * every side of its whole picture, and no share. */
const savedInPixels = (pixels: number) => ({
  ...whole(),
  cropHeight: 1_000 + pixels * 2,
  cropWidth: 1_000 + pixels * 2,
  cropX: -pixels,
  cropY: -pixels,
});

const padding = (settings: ScreenshotOutputSettings) => {
  const { crop, sourceCrop } = screenshotLayout(source, settings);
  return (crop.width - sourceCrop.width) / 2;
};

describe("a layer's padding saved in pixels", () => {
  it("reads as the share its picture gives it, looking the same", () => {
    const old = savedInPixels(45);
    expect(screenshotInsetShare(old, source)).toBeCloseTo(0.045);
    // Padded again by that share, the frame is where it was.
    const again = insetScreenshotPadding(
      old,
      source,
      screenshotInsetShare(old, source),
    );
    expect(padding(again)).toBeCloseTo(45);
  });

  it("reads as the look whose pixels it has, exactly", () => {
    // The island gave Tight, Medium and Large as rounded pixels of the
    // picture's shorter side.
    expect(screenshotInsetShare(savedInPixels(30), source)).toBe(0.03);
    expect(screenshotInsetShare(savedInPixels(60), source)).toBe(0.06);
    expect(screenshotInsetShare(savedInPixels(100), source)).toBe(0.1);
    expect(screenshotInsetShare(savedInPixels(0), source)).toBe(0);
  });

  it("is read from its share once it has one", () => {
    expect(
      screenshotInsetShare({ ...savedInPixels(45), inset: 0.06 }, source),
    ).toBe(0.06);
  });
});

describe("the padding's looks", () => {
  it("are four shares, each padded and read back as itself", () => {
    expect(INSET_LOOKS).toEqual([0, 0.03, 0.06, 0.1]);
    for (const look of INSET_LOOKS) {
      const padded = insetScreenshotPadding(whole(), source, look);
      expect(screenshotInsetShare(padded, source)).toBe(look);
      expect(padding(padded)).toBeCloseTo(look * 1_000);
    }
  });
});

describe("a padded layer cropped again", () => {
  it("keeps its padding in proportion to its new picture", () => {
    const padded = insetScreenshotPadding(whole(), source, 0.1);
    const cropped = commitScreenshotCrop(
      padded,
      withScreenshotSourceCrop(
        padded,
        sourceRect({ height: 0.5, width: 0.5, x: 0.25, y: 0.25 }),
      ),
      source,
    );
    expect(cropped.inset).toBe(0.1);
    // A tenth of the 500 pixels left, rather than the 100 it had.
    expect(padding(cropped)).toBeCloseTo(50);
  });

  it("keeps it in proportion as Recenter frames the content", () => {
    const padded = insetScreenshotPadding(whole(), source, 0.06);
    const recentred = recenterScreenshotContent(padded, source, {
      height: 200,
      width: 400,
      x: 100,
      y: 100,
    });
    expect(recentred.inset).toBe(0.06);
    expect(padding(recentred)).toBeCloseTo(0.06 * 200);
  });
});
