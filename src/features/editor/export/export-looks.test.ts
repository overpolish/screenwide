// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { lookQuality, qualityLook } from "./export-looks";

const RETINA = [200, 150, 100];
const CAMERA = [100, 75, 50];

describe("export looks", () => {
  it("shrinks and compresses more with each smaller look", () => {
    expect(lookQuality("original", RETINA)).toEqual({
      compression: 0,
      resolutionScalePercent: 200,
    });
    expect(lookQuality("balanced", RETINA)).toEqual({
      compression: 2,
      resolutionScalePercent: 200,
    });
    expect(lookQuality("smaller", RETINA)).toEqual({
      compression: 3,
      resolutionScalePercent: 150,
    });
    expect(lookQuality("smallest", RETINA)).toEqual({
      compression: 4,
      resolutionScalePercent: 100,
    });
  });

  it("names the look a quality matches, and none once customised", () => {
    const screen = (compression: number, resolutionScalePercent: number) => ({
      quality: { compression, resolutionScalePercent },
      scales: RETINA,
    });
    expect(qualityLook(screen(3, 150), null)).toBe("smaller");
    expect(qualityLook(screen(1, 200), null)).toBeNull();
  });

  it("takes a separate camera's own quality into the match", () => {
    const screen = { quality: lookQuality("balanced", RETINA), scales: RETINA };
    const camera = (look: "balanced" | "smallest") => ({
      quality: lookQuality(look, CAMERA),
      scales: CAMERA,
    });
    expect(qualityLook(screen, camera("balanced"))).toBe("balanced");
    expect(qualityLook(screen, camera("smallest"))).toBeNull();
  });
});
