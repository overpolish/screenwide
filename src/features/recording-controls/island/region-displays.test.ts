// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { regionDisplays } from "./region-displays";

import type { MonitorDetails } from "../../../bindings/MonitorDetails";

const display = (
  name: string,
  {
    height,
    width,
    x,
    y,
  }: { height: number; width: number; x: number; y: number },
): MonitorDetails => ({
  id: x * 10_000 + y,
  isBuiltin: false,
  isPrimary: false,
  layoutPosition: { x, y },
  layoutSize: { height, width },
  name,
  physicalPosition: { x, y },
  physicalSize: { height, width },
  position: { x, y },
  scaleFactor: 1,
  size: { height, width },
});

const builtIn = display("Built-in Display", {
  height: 1169,
  width: 1800,
  x: 0,
  y: 0,
});
const above = display("Studio Display", {
  height: 1080,
  width: 1920,
  x: -59,
  y: -1080,
});
const beside = display("Side Display", {
  height: 1080,
  width: 1920,
  x: 1800,
  y: 0,
});
const displays = [builtIn, above, beside];

const names = (region: Parameters<typeof regionDisplays>[0]) =>
  regionDisplays(region, builtIn, displays).map(({ name }) => name);

describe("regionDisplays", () => {
  it("names every display a region crosses onto, in their arrangement", () => {
    expect(
      names({
        position: { x: 1600, y: 100 },
        size: { height: 300, width: 400 },
      }),
    ).toEqual(["Built-in Display", "Side Display"]);
  });

  it("does not count a display the region only touches", () => {
    expect(
      names({
        position: { x: 1400, y: 0 },
        size: { height: 300, width: 400 },
      }),
    ).toEqual(["Built-in Display"]);
  });

  it("reaches displays above its owner, past its own top edge", () => {
    expect(
      names({
        position: { x: 200, y: -200 },
        size: { height: 400, width: 600 },
      }),
    ).toEqual(["Built-in Display", "Studio Display"]);
  });
});
