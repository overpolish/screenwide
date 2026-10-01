// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { applyScreenshotCropGesture } from "./screenshot-crop";
import { sourceRect } from "./screenshot-geometry";
import {
  defaultScreenshotOutput,
  screenshotSourceCrop,
  withScreenshotSourceCrop,
} from "./screenshot-output-settings";

const source = { height: 1_000, width: 1_000 };
const CENTERED_EDGE = 1 << 16;

describe("crop draw", () => {
  // The Begin anchor is measured from the crop being replaced, not from the
  // image, so a crop away from the origin is what exposes a misplaced anchor.
  const settings = withScreenshotSourceCrop(
    defaultScreenshotOutput(1_000, 1_000),
    sourceRect({ height: 0.5, width: 0.5, x: 0.2, y: 0.2 }),
  );
  const draw = (deltaX: number, deltaY: number, edges: number) =>
    screenshotSourceCrop(
      applyScreenshotCropGesture({
        anchor: { x: 0.6, y: 0.6 },
        deltaX,
        deltaY,
        edges,
        operation: "cropDraw",
        output: source,
        settings,
        source,
      }),
    );

  it("spans from the anchor to the far corner in any direction", () => {
    const drawn = draw(-0.2, -0.1, 1 | 4);
    expect(drawn.x).toBeCloseTo(0.6);
    expect(drawn.y).toBeCloseTo(0.7);
    expect(drawn.width).toBeCloseTo(0.2);
    expect(drawn.height).toBeCloseTo(0.1);
  });

  it("grows about the anchor when centered", () => {
    const drawn = draw(0.1, 0.05, 2 | 8 | CENTERED_EDGE);
    expect(drawn.x).toBeCloseTo(0.7);
    expect(drawn.y).toBeCloseTo(0.75);
    expect(drawn.width).toBeCloseTo(0.2);
    expect(drawn.height).toBeCloseTo(0.1);
  });
});
