// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { packLanes, renderOrder, restack, slotAt } from "./annotation-lanes";

const at = (id: string, lane: number, [start, end]: [number, number]) => ({
  end,
  id,
  lane,
  start,
});

// Arrow in front of Counter and Text, which it overlaps; Highlight, Focus
// and Redact apart from it in time, one in each lane.
const STACK = [
  at("arrow", 0, [10, 30]),
  at("highlight", 0, [60, 65]),
  at("counter", 1, [26, 30]),
  at("focus", 1, [50, 70]),
  at("text", 2, [0, 33]),
  at("redact", 2, [45, 60]),
];

describe("slotAt", () => {
  it("joins a lane when the middle of the bar is over it and it fits", () => {
    expect(slotAt(STACK, "highlight", 2.5)).toEqual({ isNew: false, lane: 2 });
  });

  it("opens a lane over one the bar would overlap something in", () => {
    // Arrow overlaps Counter in lane 1: the upper half opens a lane above.
    expect(slotAt(STACK, "arrow", 1.4)).toEqual({ isNew: true, lane: 1 });
    expect(slotAt(STACK, "arrow", 1.6)).toEqual({ isNew: true, lane: 2 });
  });

  it("opens a lane near the edge between two lanes", () => {
    expect(slotAt(STACK, "highlight", 2.1)).toEqual({ isNew: true, lane: 2 });
  });

  it("opens a lane above the top and below the bottom", () => {
    expect(slotAt(STACK, "text", -0.2)).toEqual({ isNew: true, lane: 0 });
    expect(slotAt(STACK, "arrow", 3)).toEqual({ isNew: true, lane: 3 });
  });
});

describe("packLanes", () => {
  it("lifts each annotation just under those in front that it overlaps", () => {
    // Text overlaps only Arrow, so it rises to the lane under it, beside
    // Counter, which it never shows with.
    const loose = [
      at("arrow", 0, [10, 30]),
      at("counter", 1, [26, 30]),
      at("text", 3, [0, 20]),
    ];
    expect(packLanes(loose)).toEqual({ arrow: 0, counter: 1, text: 1 });
  });

  it("leaves a packed stack as it is", () => {
    expect(packLanes(STACK)).toEqual(
      Object.fromEntries(STACK.map(({ id, lane }) => [id, lane])),
    );
  });
});

describe("restack", () => {
  it("keeps the order a bar dragged under two has against what it overlaps", () => {
    // Arrow goes behind Counter and in front of Text; Counter, with nothing
    // in front of it now, rises beside Highlight.
    expect(restack(STACK, "arrow", { isNew: true, lane: 2 })).toEqual({
      arrow: 1,
      counter: 0,
      focus: 1,
      highlight: 0,
      redact: 2,
      text: 2,
    });
  });

  it("puts a bar dragged into a new top lane beside those it never meets", () => {
    const pair = [at("highlight", 0, [40, 45]), at("arrow", 1, [10, 30])];
    expect(restack(pair, "arrow", { isNew: true, lane: 0 })).toEqual({
      arrow: 0,
      highlight: 0,
    });
  });

  it("closes the lane a bar leaves empty", () => {
    const pair = [at("top", 0, [0, 10]), at("bottom", 1, [0, 10])];
    expect(restack(pair, "top", { isNew: true, lane: 2 })).toEqual({
      bottom: 0,
      top: 1,
    });
  });

  it("leaves the stack alone when a bar joins its own lane", () => {
    expect(restack(STACK, "focus", { isNew: false, lane: 1 })).toEqual(
      Object.fromEntries(STACK.map(({ id, lane }) => [id, lane])),
    );
  });
});

describe("renderOrder", () => {
  it("draws the bottom lane first and the top lane last", () => {
    expect(renderOrder(STACK).map(({ id }) => id)).toEqual([
      "text",
      "redact",
      "counter",
      "focus",
      "arrow",
      "highlight",
    ]);
  });
});
