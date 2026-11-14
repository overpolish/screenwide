// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { recordedTime, timelineScale, type TimeMap } from "../timeline-scale";

import { laneLayout } from "./strip-lane-time";

// Ten recorded seconds with a 2 s gap played at 4 s, its empty source kept
// at 100 s: 0 to 4, then the gap, then 4 to 10 played 6 to 12.
const GAPPED: TimeMap = {
  duration: 12,
  toPlayed: (recorded) =>
    recorded < 4 ? recorded : recorded >= 100 ? recorded - 96 : recorded + 2,
  toRecorded: (played) =>
    played < 4 ? played : played < 6 ? played + 96 : played - 2,
};

const lanes = {
  annotations: [
    { end: 8, id: "across", kind: "arrow" as const, lane: 0, start: 3 },
  ],
  audio: undefined,
  beats: [
    {
      beat: { kind: "gap" as const },
      id: "gap-1",
      span: { end: 102, start: 100 },
    },
  ],
  map: GAPPED,
  runs: [
    { end: 4, start: 0 },
    { end: 6, start: 4 },
    { end: 12, start: 6 },
  ],
  scenes: [],
};

const rounded = (spans: { end: number; id: string; start: number }[]) =>
  spans.map(({ end, id, start }) => ({
    end: Number(end.toFixed(3)),
    id,
    start: Number(start.toFixed(3)),
  }));

describe("the strip's lanes", () => {
  it("lay a bar across a gap out as one piece, where it plays", () => {
    const layout = laneLayout(lanes, {
      edges: [],
      shown: timelineScale(GAPPED),
    });
    expect(rounded(layout.annotations)).toEqual([
      { end: 8, id: "across", start: 3 },
    ]);
    expect(layout.gaps).toEqual([{ end: 6, start: 4 }]);
    expect(layout.scale.toX(8)).toBeCloseTo(8 / 12);
  });

  it("lay a bar out where its footage plays while a move is dragged", () => {
    const shown = timelineScale(recordedTime(12));
    const layout = laneLayout(lanes, { edges: [5], shown });
    expect(layout.scale).toBe(shown);
    expect(rounded(layout.annotations)).toEqual([
      { end: 4, id: "across", start: 3 },
      { end: 102, id: "across:1", start: 100 },
      { end: 5, id: "across:2", start: 4 },
      { end: 6, id: "across:2:1", start: 5 },
    ]);
  });
});
