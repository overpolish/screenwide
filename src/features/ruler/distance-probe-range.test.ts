// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { combineDistanceProbes } from "./distance-probe-range";

describe("combineDistanceProbes", () => {
  it("joins adjacent horizontal spans when dragging forward", () => {
    expect(
      combineDistanceProbes({
        endPoint: { x: 25, y: 8 },
        endProbe: { axis: "x", end: 30, position: 8, start: 20 },
        startPoint: { x: 5, y: 8 },
        startProbe: { axis: "x", end: 10, position: 8, start: 0 },
      }),
    ).toEqual({ axis: "x", end: 30, position: 8, start: 0 });
  });

  it("joins adjacent vertical spans when dragging backward", () => {
    expect(
      combineDistanceProbes({
        endPoint: { x: 4, y: 5 },
        endProbe: { axis: "y", end: 10, position: 4, start: 0 },
        startPoint: { x: 4, y: 25 },
        startProbe: { axis: "y", end: 30, position: 4, start: 20 },
      }),
    ).toEqual({ axis: "y", end: 30, position: 4, start: 0 });
  });
});
