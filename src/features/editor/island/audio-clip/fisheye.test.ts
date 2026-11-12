// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { carried, fisheye, windowPx } from "./fisheye";

// Ten seconds of a ten-minute song on a track 300 px wide, its window
// standing 40 px in.
const WINDOW = windowPx(300, 10_000, 600_000);
const lens = fisheye({
  fromMs: 60_000,
  soundMs: 600_000,
  startX: 40,
  toMs: 70_000,
  width: 300,
  window: WINDOW,
});

describe("the clip island's fisheye", () => {
  it("gives a short clip a good share of the width wherever it stands", () => {
    expect(WINDOW).toBe(120);
    expect([lens.toX(60_000), lens.toX(70_000)]).toEqual([40, 160]);
    expect(lens.toX(0)).toBeCloseTo(0);
    expect(lens.toX(600_000)).toBeCloseTo(300);
  });

  it("lets the zoom fall away gradually past the window's edge", () => {
    const pxPerSecond = (ms: number) => lens.toX(ms + 500) - lens.toX(ms - 500);
    const inside = pxPerSecond(65_000);
    // Just past the edge nearly as zoomed as inside, then less and less.
    expect(pxPerSecond(71_000)).toBeGreaterThan(inside * 0.7);
    expect(pxPerSecond(76_000)).toBeLessThan(pxPerSecond(71_000));
    expect(pxPerSecond(300_000)).toBeLessThan(pxPerSecond(76_000));
  });

  it("finds again the moment it draws, inside and either side of the window", () => {
    for (const ms of [0, 30_000, 64_000, 75_000, 300_000, 600_000])
      expect(lens.toMs(lens.toX(ms))).toBeCloseTo(ms, -1);
  });
});

describe("carrying the window", () => {
  const way = { finePerPx: 10_000 / 120, remainingMs: 530_000, roomPx: 140 };

  it("moves finely near where it was taken", () => {
    expect(carried({ ...way, px: 4 })).toBeLessThan(400);
  });

  it("reaches the sound's end as the window reaches the track's", () => {
    expect(carried({ ...way, px: 140 })).toBeCloseTo(530_000);
    expect(carried({ ...way, px: 200 })).toBeCloseTo(530_000);
  });

  it("moves finely on against the track's end with sound still beyond", () => {
    expect(carried({ ...way, px: 6, roomPx: 0 })).toBeCloseTo(500);
  });
});
