// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { sourceRect } from "../screenshot-geometry";

import { centredSourceCrop } from "./crop-target";

const source = { height: 1000, width: 2000 };

const rect = (value: {
  height: number;
  width: number;
  x: number;
  y: number;
}) => sourceRect(value);

describe("centredSourceCrop", () => {
  it("grows and shrinks about the crop's own centre", () => {
    const current = rect({ height: 0.5, width: 0.5, x: 0.25, y: 0.25 });
    // Centre (0.5, 0.5); half the size, so a quarter of the source each way.
    const next = centredSourceCrop(
      current,
      { height: 250, width: 500 },
      source,
    );
    expect(next.width).toBeCloseTo(0.25);
    expect(next.height).toBeCloseTo(0.25);
    expect(next.x).toBeCloseTo(0.375);
    expect(next.y).toBeCloseTo(0.375);
  });

  it("slides back inside the source rather than shrinking at an edge", () => {
    // A crop hard against the top left, asked to double in size: it keeps the
    // size it was given and moves in off the edge.
    const current = rect({ height: 0.2, width: 0.2, x: 0, y: 0 });
    const next = centredSourceCrop(
      current,
      { height: 800, width: 1600 },
      source,
    );
    expect(next.width).toBeCloseTo(0.8);
    expect(next.height).toBeCloseTo(0.8);
    expect(next.x).toBe(0);
    expect(next.y).toBe(0);

    const trailing = centredSourceCrop(
      rect({ height: 0.2, width: 0.2, x: 0.8, y: 0.8 }),
      { height: 800, width: 1600 },
      source,
    );
    expect(trailing.x).toBeCloseTo(0.2);
    expect(trailing.y).toBeCloseTo(0.2);
  });

  it("clamps a size past the source to the whole source", () => {
    const next = centredSourceCrop(
      rect({ height: 0.2, width: 0.2, x: 0.4, y: 0.4 }),
      { height: 4000, width: 9000 },
      source,
    );
    expect(next).toEqual(rect({ height: 1, width: 1, x: 0, y: 0 }));
  });

  it("keeps at least one source pixel", () => {
    const next = centredSourceCrop(
      rect({ height: 0.5, width: 0.5, x: 0.25, y: 0.25 }),
      { height: 0, width: -10 },
      source,
    );
    expect(next.width).toBeCloseTo(1 / source.width);
    expect(next.height).toBeCloseTo(1 / source.height);
  });
});
