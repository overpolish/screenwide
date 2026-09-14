// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { randomizeMeshBackground } from "./background-random";

describe("mesh randomization", () => {
  it("keeps the palette, locks, generator, and warp while changing arrangement data", () => {
    const mesh = {
      colors: ["#111111", "#222222", "#333333"],
      generator: "ribbons",
      kind: "mesh" as const,
      lockedColors: [true, false, true],
      points: [],
      seed: 10,
      warpPercent: 12,
    };

    const next = randomizeMeshBackground(mesh);

    expect(next.colors).toEqual(mesh.colors);
    expect(next.lockedColors).toEqual(mesh.lockedColors);
    expect(next.generator).toBe(mesh.generator);
    expect(next.warpPercent).toBe(mesh.warpPercent);
    expect(next.seed).toBeGreaterThan(0);
  });
});
