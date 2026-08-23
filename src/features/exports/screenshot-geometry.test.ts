// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  canvasRect,
  scaleCanvasGeometry,
  sourceRect,
  translateSourceRect,
  translateCanvasGeometry,
} from "./screenshot-geometry";

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

describe("screenshot source geometry", () => {
  it("constrains source rectangles to the source image", () => {
    expect(sourceRect({ height: 1.4, width: 1.5, x: -0.2, y: -0.3 })).toEqual({
      height: 1,
      width: 1,
      x: 0,
      y: 0,
    });
  });

  it("rejects non-finite and empty source rectangles", () => {
    expect(() =>
      sourceRect({ height: 0.5, width: Number.NaN, x: 0, y: 0 }),
    ).toThrow("finite");
    expect(() => sourceRect({ height: 0.5, width: 0.2, x: 1.2, y: 0 })).toThrow(
      "overlap",
    );
  });

  it("clamps crop movement without changing its size", () => {
    expectRectClose(
      translateSourceRect(
        sourceRect({ height: 0.4, width: 0.3, x: 0.2, y: 0.3 }),
        { x: 2, y: -2 },
      ),
      sourceRect({ height: 0.4, width: 0.3, x: 0.7, y: 0 }),
    );
  });
});

describe("screenshot canvas geometry", () => {
  const geometry = {
    frame: canvasRect({ height: 200, width: 300, x: 100, y: 50 }),
    sourceToCanvas: { scale: 2, x: 80, y: 40 },
  };

  it("moves the frame and image transform through one translation", () => {
    expect(translateCanvasGeometry(geometry, { x: 25, y: -10 })).toEqual({
      frame: { height: 200, width: 300, x: 125, y: 40 },
      sourceToCanvas: { scale: 2, x: 105, y: 30 },
    });
  });

  it("scales the frame and image transform around the same anchor", () => {
    expect(scaleCanvasGeometry(geometry, { x: 100, y: 50 }, 0.5)).toEqual({
      frame: { height: 100, width: 150, x: 100, y: 50 },
      sourceToCanvas: { scale: 1, x: 90, y: 45 },
    });
  });
});
