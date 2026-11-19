// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { INSET_LOOKS, percent, points } from "../layer-measure";

import { commitScreenshotCrop } from "./screenshot-crop";
import { sourceRect } from "./screenshot-geometry";
import {
  insetScreenshotPadding,
  screenshotInsetPoints,
} from "./screenshot-inset";
import { withSnappedMeasures } from "./screenshot-measure-snap";
import {
  defaultScreenshotOutput,
  screenshotLayout,
  ScreenshotOutputSettings,
} from "./screenshot-output";
import { withScreenshotSourceCrop } from "./screenshot-output-settings";
import { recenterScreenshotContent } from "./screenshot-recenter";

/** A Retina capture, 1000 pixels square: two pixels to the point. */
const source = { height: 1_000, scaleFactor: 2, width: 1_000 };

/** The whole capture on a canvas of its size. */
const whole = (capture = source) =>
  withScreenshotSourceCrop(
    defaultScreenshotOutput(capture.width, capture.height),
    sourceRect({ height: 1, width: 1, x: 0, y: 0 }),
  );

const padding = (
  settings: ScreenshotOutputSettings,
  capture: { height: number; width: number } = source,
) => {
  const { crop, sourceCrop } = screenshotLayout(capture, settings);
  return (crop.width - sourceCrop.width) / 2;
};

describe("the padding's looks", () => {
  it("pad by points of the capture, read back as themselves", () => {
    for (const look of INSET_LOOKS) {
      const padded = insetScreenshotPadding(whole(), source, look);
      expect(screenshotInsetPoints(padded, source)).toBe(look);
      expect(padding(padded)).toBeCloseTo(look * 2);
    }
  });

  it("pad a short strip as much as a tall capture", () => {
    const strip = { height: 120, scaleFactor: 2, width: 2_000 };
    const screen = { height: 1_200, scaleFactor: 2, width: 2_000 };
    const medium = INSET_LOOKS[2];
    expect(
      padding(insetScreenshotPadding(whole(strip), strip, medium), strip),
    ).toBeCloseTo(
      padding(insetScreenshotPadding(whole(screen), screen, medium), screen),
    );
  });
});

describe("a padded layer cropped again", () => {
  it("keeps its padding in points round its new picture", () => {
    const padded = insetScreenshotPadding(whole(), source, 64);
    const cropped = commitScreenshotCrop(
      padded,
      withScreenshotSourceCrop(
        padded,
        sourceRect({ height: 0.5, width: 0.5, x: 0.25, y: 0.25 }),
      ),
      source,
    );
    expect(cropped.inset).toEqual(points(64));
    expect(padding(cropped)).toBeCloseTo(128);
  });

  it("keeps it as Recenter frames the content", () => {
    const padded = insetScreenshotPadding(whole(), source, 32);
    const recentred = recenterScreenshotContent(padded, source, {
      height: 200,
      width: 400,
      x: 100,
      y: 100,
    });
    expect(recentred.inset).toEqual(points(32));
    expect(padding(recentred)).toBeCloseTo(64);
  });
});

describe("a layer saved before the padding was kept in points", () => {
  it("reads a frame-only padding as the look its share was nearest", () => {
    // 50 pixels round a 1000 pixel picture was a share of 0.05: Medium.
    const framed = {
      ...whole(),
      cropHeight: 1_100,
      cropWidth: 1_100,
      cropX: -50,
      cropY: -50,
    };
    expect(screenshotInsetPoints(framed, source)).toBe(32);
  });

  it("opens with its share and corners snapped to looks in points", () => {
    // As a project saved it then: a bare share and a bare percent, under the
    // names they had, and no length with a unit.
    const { inset: _inset, radius: _radius, ...current } = whole();
    const saved = {
      ...current,
      cropHeight: 1_200,
      cropWidth: 1_200,
      cropX: -100,
      cropY: -100,
      inset: 0.1,
      radiusPercent: 12,
    } as unknown as ScreenshotOutputSettings;
    const opened = withSnappedMeasures(saved, source, "points");
    expect(opened.inset).toEqual(points(64));
    expect(opened.radius).toEqual(points(24));
    expect(padding(opened)).toBeCloseTo(128);
    expect("radiusPercent" in opened).toBe(false);
  });

  it("keeps a camera's corners as a share", () => {
    const opened = withSnappedMeasures(
      { ...whole(), radius: percent(50) },
      source,
      "percent",
    );
    expect(opened.radius).toEqual(percent(50));
  });
});
