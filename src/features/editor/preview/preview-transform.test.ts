// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { previewFitWidth } from "./preview-transform";

describe("preview fit width", () => {
  it("does not upscale a small canvas", () => {
    expect(previewFitWidth(400, 200, 1_000)).toBe(416);
  });

  it("uses the available height for a large canvas", () => {
    expect(previewFitWidth(1_600, 900, 500)).toBeCloseTo(876.4444);
  });
});
