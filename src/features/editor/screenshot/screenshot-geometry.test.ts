// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { sourceRect, translateSourceRect } from "./screenshot-geometry";

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
