// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { ANNOTATION_BAR_PX } from "./annotation-bar";
import { carryScrollSpeed, heldLanes, holdStep } from "./annotation-hold";
import { type TimelineAnnotation } from "./annotation-lane";
import { packLanes } from "./annotation-lanes";

const at = (
  id: string,
  lane: number,
  [start, end]: [number, number],
): TimelineAnnotation => ({ end, id, kind: "arrow", lane, start });

// Arrow and Highlight share the top lane; Focus sits under Highlight.
const STACK = [
  at("arrow", 0, [14, 30]),
  at("highlight", 0, [46, 50]),
  at("focus", 1, [40, 62]),
];

/** Holds Arrow through each pointer height in turn, `span` its time. */
const hold = (heights: number[], span = { end: 56, start: 40 }) =>
  heights.reduce<ReturnType<typeof holdStep> | null>(
    (current, y) =>
      holdStep({
        annotations: STACK,
        current,
        hold: { guide: null, isLifted: true, span, startY: 0, y },
        id: "arrow",
        shown: packLanes(STACK),
      }),
    null,
  );

describe("holding a bar", () => {
  it("goes between two lanes without moving the lane above", () => {
    // Over Highlight's time, let go at the line between Highlight and Focus:
    // the bar's middle half a lane down from where it was.
    const held = hold([ANNOTATION_BAR_PX / 4, ANNOTATION_BAR_PX / 2]);
    expect(held && heldLanes(STACK, held)).toEqual({
      arrow: 1,
      focus: 2,
      highlight: 0,
    });
  });

  it("stays in its lane for a few pixels of wobble", () => {
    const held = hold([2, -2, 3], { end: 30, start: 14 });
    expect(held && heldLanes(STACK, held)).toEqual(packLanes(STACK));
  });

  it("keeps a slot near its edge until the pointer clearly leaves it", () => {
    // Taken at the line between the lanes, then nudged back up 6 px, over
    // the bottom of its own lane, which it would join if it came from there.
    const held = hold([10, 4], { end: 30, start: 14 });
    expect(held?.slot).toEqual({ isNew: true, lane: 1 });
  });
});

describe("carrying a bar to the edge of the lanes shown", () => {
  const shown = 4 * ANNOTATION_BAR_PX;

  it("keeps still with the pointer in the first or last lane's middle", () => {
    expect(carryScrollSpeed(ANNOTATION_BAR_PX / 2, shown)).toBe(0);
    expect(carryScrollSpeed(shown - ANNOTATION_BAR_PX / 2, shown)).toBe(0);
  });

  it("scrolls toward the edge, faster nearer it, up to a top speed", () => {
    const up = [4, 0, -10, -40].map((y) => carryScrollSpeed(y, shown));
    expect(up[0]).toBeLessThan(0);
    expect(up[1]).toBeLessThan(up[0] ?? 0);
    expect(up[2]).toBeLessThan(up[1] ?? 0);
    expect(up[3]).toBe(up[2]);
    expect(carryScrollSpeed(shown + 10, shown)).toBe(-(up[2] ?? 0));
  });
});
