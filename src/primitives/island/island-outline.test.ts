// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { islandOutline } from "./island-outline";

const row = { flare: 0, height: 48, radius: 24, top: 24, width: 400 };

/** The radii of the outline's arcs, in the order the path draws them:
 * the row's two top corners, then down the end side, along the bottom, and
 * up the start side. */
const radii = (path: string) =>
  [...path.matchAll(/A([\d.]+) /g)].map((match) => Number(match[1]));

describe("islandOutline", () => {
  it("keeps a side's corner whole however little of the section shows", () => {
    // Folding, the corner rises into the row and melts into it rather than
    // shrinking to a step.
    for (const height of [1, 4, 10, 20, 35]) {
      const [, , , , endCorner, startCorner] = radii(
        islandOutline({ row, section: { height, start: 100, width: 200 } }),
      );
      expect(endCorner).toBe(22);
      expect(startCorner).toBe(22);
    }
  });

  it("does not jump where the join moves from the section's side to its corner", () => {
    // The join leaves the row and meets the section at the same points just
    // above and just below the height that holds the corner and the join.
    const joinAt = (height: number) => {
      const path = islandOutline({
        row,
        section: { height, start: 100, width: 200 },
      });
      const match = /H([\d.]+) A14 14 0 0 0 ([\d.]+) ([\d.]+)/.exec(path);
      return (match ?? []).slice(1).map(Number);
    };
    const above = joinAt(36.01);
    const below = joinAt(35.99);
    expect(above).toHaveLength(3);
    above.forEach((value, index) => {
      expect(Math.abs(value - below[index])).toBeLessThan(0.2);
    });
  });

  it("starts a side that meets the row's end at the row's own corner", () => {
    // Just opening: the corner is the row's, carried down; open, the
    // section's.
    const opening = radii(
      islandOutline({ row, section: { height: 0.5, start: 0, width: 200 } }),
    );
    expect(opening[opening.length - 1]).toBeCloseTo(row.radius, 0);
    const open = radii(
      islandOutline({ row, section: { height: 120, start: 0, width: 200 } }),
    );
    expect(open[open.length - 1]).toBe(22);
  });

  it("grows a side's join from nothing as the section leaves the row's end", () => {
    const joinAt = (start: number) => {
      const all = radii(
        islandOutline({ row, section: { height: 120, start, width: 200 } }),
      );
      return all[all.length - 2];
    };
    expect(joinAt(1)).toBe(0);
    expect(joinAt(30)).toBeGreaterThan(0);
    expect(joinAt(80)).toBe(14);
  });

  it("keeps a narrow section's corners within its width", () => {
    const [, , , , endCorner, startCorner] = radii(
      islandOutline({ row, section: { height: 120, start: 100, width: 20 } }),
    );
    expect(endCorner + startCorner).toBeLessThanOrEqual(20);
  });
});
