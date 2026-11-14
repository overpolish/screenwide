// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { scenesClearOfGaps } from "./recording-scene-gaps";

const scene = (id: string, startMs: number, endMs: number) => ({
  endMs,
  id,
  startMs,
});

describe("scenesClearOfGaps", () => {
  it("cuts a scene crossing a gap back to either side of it", () => {
    const parts: [string, string][] = [];
    const cleared = scenesClearOfGaps(
      [scene("before", 1_000, 4_000), scene("across", 7_500, 12_907)],
      [{ endMs: 11_077, startMs: 8_077 }],
      (from, part) => {
        parts.push([part.id, from.id]);
      },
    );
    expect(cleared.map(({ endMs, startMs }) => [startMs, endMs])).toEqual([
      [1_000, 4_000],
      [7_500, 8_077],
      [11_077, 12_907],
    ]);
    expect(cleared[1].id).toBe("across");
    expect(parts).toEqual([[cleared[2].id, "across"]]);
  });

  it("drops a side too short to be a scene, the other keeping its id", () => {
    const cleared = scenesClearOfGaps(
      [scene("across", 7_800, 12_000)],
      [{ endMs: 11_000, startMs: 8_000 }],
    );
    expect(cleared).toEqual([scene("across", 11_000, 12_000)]);
  });

  it("leaves scenes clear of every gap as they are", () => {
    const clips = [scene("a", 0, 2_000), scene("b", 5_000, 6_000)];
    expect(scenesClearOfGaps(clips, [{ endMs: 5_000, startMs: 2_000 }])).toBe(
      clips,
    );
  });
});
