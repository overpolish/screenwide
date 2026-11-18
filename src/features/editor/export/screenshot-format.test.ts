// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { screenshotFormatChoice } from "./screenshot-format";

describe("screenshotFormatChoice", () => {
  it("offers every format for a screenshot each one holds", () => {
    expect(
      screenshotFormatChoice("webp", { height: 16_383, width: 1_000 }),
    ).toEqual({ selected: "webp", unavailable: [] });
  });

  it("saves a scrolling capture too tall for WebP as a PNG", () => {
    expect(
      screenshotFormatChoice("webp", { height: 30_000, width: 1_048 }),
    ).toEqual({ selected: "png", unavailable: ["webp"] });
  });

  it("keeps JPEG until the picture outgrows it too", () => {
    expect(
      screenshotFormatChoice("jpeg", { height: 30_000, width: 1_048 }),
    ).toEqual({ selected: "jpeg", unavailable: ["webp"] });
    expect(
      screenshotFormatChoice("jpeg", { height: 70_000, width: 1_048 }),
    ).toEqual({ selected: "png", unavailable: ["jpeg", "webp"] });
  });
});
