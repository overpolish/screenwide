// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  angleAtPoint,
  signedAngle,
  snapAngle,
  wrapAngle,
} from "./angle-geometry";

const dial = { height: 24, left: 100, top: 200, width: 24 };
const at = (clientX: number, clientY: number) =>
  angleAtPoint(dial, { clientX, clientY });

describe("angle dial geometry", () => {
  it("folds any angle into one turn", () => {
    expect(wrapAngle(0)).toBe(0);
    expect(wrapAngle(360)).toBe(0);
    expect(wrapAngle(-90)).toBe(270);
    expect(wrapAngle(725)).toBe(5);
  });

  it("folds an angle into the half turn either side of zero", () => {
    expect(signedAngle(0)).toBe(0);
    expect(signedAngle(90)).toBe(90);
    expect(signedAngle(180)).toBe(-180);
    expect(signedAngle(270)).toBe(-90);
    expect(signedAngle(-450)).toBe(-90);
    expect(signedAngle(725)).toBe(5);
  });

  it("snaps to a step without landing on a full turn", () => {
    expect(snapAngle(20, 45)).toBe(0);
    expect(snapAngle(200, 45)).toBe(180);
    expect(snapAngle(350, 45)).toBe(0);
    expect(snapAngle(359.6, 1)).toBe(0);
    expect(snapAngle(-1, 1)).toBe(359);
  });

  it("reads a press as a bearing clockwise from twelve o'clock", () => {
    expect(at(112, 200)).toBe(0);
    expect(at(124, 212)).toBe(90);
    expect(at(112, 224)).toBe(180);
    expect(at(100, 212)).toBe(270);
    expect(at(124, 200)).toBeCloseTo(45, 6);
  });

  it("reads no bearing from a press on the centre", () => {
    expect(at(112, 212)).toBeNull();
  });
});
