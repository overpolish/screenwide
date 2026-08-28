// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { labelKeyForLine } from "./ruler-label-target";
import { Guide } from "./ruler-types";

const viewport = { height: 400, width: 600 };
const guides: readonly Guide[] = [
  { anchor: 80, axis: "x", id: 1, position: 100 },
  { anchor: 100, axis: "x", id: 2, position: 180 },
  { anchor: 300, axis: "x", id: 3, position: 260 },
];

describe("labelKeyForLine", () => {
  it("maps measurements and probes to their own labels", () => {
    expect(
      labelKeyForLine({
        cursor: { x: 0, y: 0 },
        guides,
        selected: { id: 4, kind: "measurement" },
        viewport,
      }),
    ).toBe("m4");
    expect(
      labelKeyForLine({
        cursor: { x: 0, y: 0 },
        guides,
        selected: { id: 7, kind: "probe" },
        viewport,
      }),
    ).toBe("p7");
    expect(
      labelKeyForLine({
        cursor: { x: 0, y: 0 },
        guides,
        selected: { id: 9, kind: "radius" },
        viewport,
      }),
    ).toBe("r9");
  });

  it("chooses the nearest adjacent gap when a guide is clicked", () => {
    expect(
      labelKeyForLine({
        cursor: { x: 180, y: 90 },
        guides,
        selected: { id: 2, kind: "guide" },
        viewport,
      }),
    ).toBe("gx:1-2");
    expect(
      labelKeyForLine({
        cursor: { x: 180, y: 290 },
        guides,
        selected: { id: 2, kind: "guide" },
        viewport,
      }),
    ).toBe("gx:2-3");
  });
});
