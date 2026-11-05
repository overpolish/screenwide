// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { partId } from "../strip/strip-part";

import { movedScene } from "./scene-move";

const scenes = [
  { end: 4, id: "a", start: 2 },
  { end: 10, id: "b", start: 6 },
  { end: 16, id: "c", start: 14 },
];
const moving = { end: 10, id: "b", start: 6 };

describe("movedScene", () => {
  it("moves a scene whole, stopping at its neighbours", () => {
    expect(movedScene({ by: 1, duration: 20, moving, scenes }).span).toEqual({
      end: 11,
      start: 7,
    });
    expect(movedScene({ by: -5, duration: 20, moving, scenes }).span).toEqual({
      end: 8,
      start: 4,
    });
    expect(movedScene({ by: 9, duration: 20, moving, scenes }).span).toEqual({
      end: 14,
      start: 10,
    });
  });

  it("keeps a lone scene within the recording", () => {
    const lone = [moving];
    expect(
      movedScene({ by: 20, duration: 12, moving, scenes: lone }).span,
    ).toEqual({ end: 12, start: 8 });
  });

  it("is not stopped by its own parts", () => {
    const parts = [
      { end: 10, id: partId("b", 0), start: 6 },
      { end: 13, id: partId("b", 1), start: 11 },
    ];
    expect(
      movedScene({
        by: 2,
        duration: 20,
        moving: parts[0] ?? moving,
        scenes: parts,
      }).span,
    ).toEqual({ end: 12, start: 8 });
  });

  it("snaps its start, or else its end, and drops a snap it has no room for", () => {
    const snapTo =
      (...edges: number[]) =>
      (seconds: number) => {
        const edge = edges.find((each) => Math.abs(each - seconds) < 0.5);
        return edge === undefined
          ? { isSnapped: false, seconds }
          : { isSnapped: true, seconds: edge };
      };
    expect(
      movedScene({ by: 0.8, duration: 20, moving, scenes }, snapTo(7)),
    ).toEqual({ guide: 7, span: { end: 11, start: 7 } });
    expect(
      movedScene({ by: 0.8, duration: 20, moving, scenes }, snapTo(11)),
    ).toEqual({ guide: 11, span: { end: 11, start: 7 } });
    expect(
      movedScene({ by: 3.8, duration: 20, moving, scenes }, snapTo(14.2)),
    ).toEqual({ guide: null, span: { end: 14, start: 10 } });
  });
});
