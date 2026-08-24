// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { rulerWheelZoomFactor } from "./ruler-zoom";

describe("rulerWheelZoomFactor", () => {
  it("matches the export preview for a Windows wheel notch", () => {
    expect(rulerWheelZoomFactor(-100, false)).toBeCloseTo(Math.exp(0.12));
  });

  it("retains the fine-grained macOS response", () => {
    expect(rulerWheelZoomFactor(-1, true)).toBeCloseTo(Math.exp(0.01));
  });
});
