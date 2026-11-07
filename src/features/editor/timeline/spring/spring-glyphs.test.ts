// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { glyphPlaces } from "./spring-glyphs";

describe("glyphPlaces", () => {
  it("gives each stretch its own glyph where there is room between them", () => {
    expect(glyphPlaces([200, 40], 400)).toEqual([40, 200]);
  });

  it("shares one glyph between stretches too close for two", () => {
    expect(glyphPlaces([100, 110, 120, 300], 400)).toEqual([110, 300]);
  });

  it("keeps half the coil by standing one glyph in the middle of them all", () => {
    // Four glyphs a glyph's room apart would take out 120 px of 200.
    expect(glyphPlaces([20, 60, 100, 140], 200)).toEqual([80]);
  });
});
