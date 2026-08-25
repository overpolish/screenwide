// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  magnifierCapturePoint,
  magnifierHandlePoint,
} from "./magnifier-geometry";

describe("magnifierHandlePoint", () => {
  const rect = { height: 300, width: 400, x: 100, y: 200 };

  it("uses current region geometry on the moving axis", () => {
    expect(magnifierHandlePoint(rect, "right", { x: 496, y: 275 })).toEqual({
      x: 500,
      y: 275,
    });
  });

  it("uses exact region edges for a corner", () => {
    expect(
      magnifierHandlePoint(rect, "bottomLeft", { x: 104, y: 496 }),
    ).toEqual({ x: 100, y: 500 });
  });
});

describe("magnifierCapturePoint", () => {
  it("maps Retina WebView points to capture pixels", () => {
    expect(
      magnifierCapturePoint(
        { x: 320, y: 180 },
        { height: 900, width: 1600 },
        { height: 1800, width: 3200 },
      ),
    ).toEqual({ x: 640, y: 360 });
  });

  it("uses the live viewport ratio independently on each axis", () => {
    expect(
      magnifierCapturePoint(
        { x: 480, y: 270 },
        { height: 720, width: 1280 },
        { height: 1080, width: 1920 },
      ),
    ).toEqual({ x: 720, y: 405 });
  });
});
