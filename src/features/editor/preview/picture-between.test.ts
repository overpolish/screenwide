// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../screenshot/screenshot-output";

import { colourBetween, outputBetween } from "./picture-between";

const plain = { ...defaultScreenshotOutput(800, 600), dropShadow: false };

describe("outputBetween", () => {
  it("starts on the old look and ends on the new one", () => {
    const to = { ...plain, cropWidth: plain.cropWidth + 40, radiusPercent: 20 };
    expect(outputBetween(plain, to, 0).radiusPercent).toBe(plain.radiusPercent);
    expect(outputBetween(plain, to, 0).cropWidth).toBe(plain.cropWidth);
    expect(outputBetween(plain, to, 1).radiusPercent).toBe(20);
    expect(outputBetween(plain, to, 0.5).radiusPercent).toBe(
      (plain.radiusPercent + 20) / 2,
    );
  });

  it("fades a shadow in and out rather than switching it", () => {
    const shadowed = { ...plain, dropShadow: true };
    const appearing = outputBetween(plain, shadowed, 0.25);
    expect(appearing.dropShadow).toBe(true);
    expect(appearing.dropShadowStrength).toBe(0.25);
    const going = outputBetween(shadowed, plain, 0.25);
    expect(going.dropShadow).toBe(true);
    expect(going.dropShadowStrength).toBe(0.75);
    expect(outputBetween(plain, shadowed, 0).dropShadow).toBe(false);
  });

  it("fades from a shadow caught part of the way", () => {
    const halfway = { ...plain, dropShadow: true, dropShadowStrength: 0.5 };
    const back = outputBetween(halfway, plain, 0.5);
    expect(back.dropShadowStrength).toBe(0.25);
  });

  it("keeps the padding's colour while the padding shrinks away", () => {
    const padded = { ...plain, recenterInsetColor: "#112233" };
    expect(outputBetween(padded, plain, 0.5).recenterInsetColor).toBe(
      "#112233",
    );
    expect(outputBetween(plain, padded, 0.5).recenterInsetColor).toBe(
      "#112233",
    );
  });

  it("grows a turn out of upright and settles it back", () => {
    const turned = { ...plain, rotation: { x: 10, y: -20, z: 4 } };
    expect(outputBetween(plain, turned, 0.5).rotation).toEqual({
      x: 5,
      y: -10,
      z: 2,
    });
    expect(outputBetween(turned, plain, 0.5).rotation).toEqual({
      x: 5,
      y: -10,
      z: 2,
    });
    expect(outputBetween(plain, plain, 0.5).rotation).toBeUndefined();
  });

  it("keeps the canvas in whole pixels", () => {
    const larger = { ...plain, height: 601, width: 801 };
    const between = outputBetween(plain, larger, 0.3);
    expect(Number.isInteger(between.width)).toBe(true);
    expect(Number.isInteger(between.height)).toBe(true);
  });
});

describe("colourBetween", () => {
  it("mixes two colours and leaves any other to the new one", () => {
    expect(colourBetween("#000000", "#ffffff", 0.5)).toBe("#808080");
    expect(colourBetween("#000000", "#ffffff", 0)).toBe("#000000");
    expect(colourBetween("transparent", "#ffffff", 0.5)).toBe("#ffffff");
  });
});
