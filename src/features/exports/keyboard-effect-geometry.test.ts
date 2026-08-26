// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  keyboardMaximumSizePercent,
  keyboardSelectionGeometry,
} from "./keyboard-effect-geometry";

describe("keyboardMaximumSizePercent", () => {
  it("reserves frame margin and animation headroom for long chords", () => {
    expect(
      keyboardMaximumSizePercent({
        height: 1080,
        maximumWidthUnits: 138,
        width: 1920,
      }),
    ).toBe(365);
  });

  it("keeps the 500 percent product ceiling when there is room", () => {
    expect(
      keyboardMaximumSizePercent({
        height: 2160,
        maximumWidthUnits: 50,
        width: 3840,
      }),
    ).toBe(500);
  });
});

describe("keyboardSelectionGeometry", () => {
  it("fits oversized overrides and exposes the same resize limits as appearance", () => {
    const geometry = keyboardSelectionGeometry({
      height: 1080,
      maximumWidthUnits: 138,
      position: { centerX: 0.5, centerY: 0.5, sizePercent: 500 },
      sizePercent: 100,
      width: 1920,
    });

    expect(geometry?.sizePercent).toBe(365);
    expect(geometry?.maximumSizePercent).toBe(365);
    expect(geometry?.minimumSizePercent).toBe(50);
  });
});
