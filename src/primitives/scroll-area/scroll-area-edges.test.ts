// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { getEdgeOpacities } from "./scroll-area-edges";

describe("scroll area edges", () => {
  it("hides effects without overflow or when disabled", () => {
    expect(getEdgeOpacities(0, 0, { effect: "shadow" })).toEqual({
      end: 0,
      start: 0,
    });
    expect(getEdgeOpacities(20, 100, { effect: "none" })).toEqual({
      end: 0,
      start: 0,
    });
  });
  it("preserves the default shadow fade", () => {
    expect(getEdgeOpacities(25, 100, { effect: "shadow" })).toEqual({
      end: 0.75,
      start: 0.25,
    });
  });
  it("hides the shadow at the corresponding boundary", () => {
    expect(getEdgeOpacities(0, 100, { effect: "shadow" })).toEqual({
      end: 1,
      start: 0,
    });
    expect(getEdgeOpacities(50, 100, { effect: "shadow" })).toEqual({
      end: 0.5,
      start: 0.5,
    });
    expect(getEdgeOpacities(100, 100, { effect: "shadow" })).toEqual({
      end: 0,
      start: 1,
    });
    expect(getEdgeOpacities(12, 100, { effect: "shadow" }).start).toBe(0.12);
  });
  it("clamps overscroll at both boundaries", () => {
    expect(getEdgeOpacities(-10, 100, { effect: "shadow" })).toEqual({
      end: 1,
      start: 0,
    });
    expect(getEdgeOpacities(110, 100, { effect: "shadow" })).toEqual({
      end: 0,
      start: 1,
    });
  });
});
