// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { getEdgeFades } from "./scroll-area-edges";

const fade = { effect: "fade", length: 16 } as const;

describe("scroll area edges", () => {
  it("fades nothing without overflow or when turned off", () => {
    expect(getEdgeFades(0, 0, fade)).toEqual({ end: 0, start: 0 });
    expect(getEdgeFades(20, 100, { effect: "none", length: 16 })).toEqual({
      end: 0,
      start: 0,
    });
  });
  it("fades only the ends with more beyond them", () => {
    expect(getEdgeFades(0, 100, fade)).toEqual({ end: 16, start: 0 });
    expect(getEdgeFades(50, 100, fade)).toEqual({ end: 16, start: 16 });
    expect(getEdgeFades(100, 100, fade)).toEqual({ end: 0, start: 16 });
  });
  it("grows a fade over the first stretch scrolled from its end", () => {
    expect(getEdgeFades(6, 100, fade)).toEqual({ end: 16, start: 6 });
    expect(getEdgeFades(96, 100, fade)).toEqual({ end: 4, start: 16 });
  });
  it("clamps overscroll at both ends", () => {
    expect(getEdgeFades(-10, 100, fade)).toEqual({ end: 16, start: 0 });
    expect(getEdgeFades(110, 100, fade)).toEqual({ end: 0, start: 16 });
  });
});
