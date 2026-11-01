// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { recordedTime, type TimeMap, timelineScale } from "../timeline-scale";

import { edgeSnap, stripSnapTargets } from "./strip-snap";

// A strip 100 seconds long, 1000 px wide: a second is 10 px, and an end
// snaps within 8 px, under a second.
const WIDTH = 1000;
const scale = timelineScale(recordedTime(100));

describe("the strip's snap targets", () => {
  it("hold every part of the annotations and scenes, by the item it is part of, the beats, lulls, cuts and playhead", () => {
    expect(
      stripSnapTargets({
        annotations: [
          { end: 30, id: "arrow", start: 10 },
          { end: 45, id: "arrow:1", start: 40 },
        ],
        beats: [{ span: { end: 5, start: 0 } }],
        cuts: [{ end: 60, start: 55 }],
        lulls: [{ span: { end: 8, start: 5 } }],
        playheadAt: 22,
        scenes: [{ end: 90, id: "scene", start: 70 }],
      }),
    ).toEqual([
      { end: 30, owner: "arrow", start: 10 },
      { end: 45, owner: "arrow", start: 40 },
      { end: 90, owner: "scene", start: 70 },
      { end: 5, start: 0 },
      { end: 8, start: 5 },
      { end: 60, start: 55 },
      { end: 22, start: 22 },
    ]);
  });

  it("leave the playhead out where it is not known", () => {
    expect(stripSnapTargets({ playheadAt: null })).toEqual([]);
  });
});

describe("an end dragged with the strip's snap", () => {
  const targets = stripSnapTargets({
    annotations: [{ end: 30, id: "arrow", start: 10 }],
    scenes: [{ end: 50, id: "scene", start: 40 }],
  });

  it("settles on an annotation's start or end in reach", () => {
    const snap = edgeSnap({ scale, targets, width: WIDTH });
    expect(snap(10.5)).toEqual({ isSnapped: true, seconds: 10 });
    expect(snap(29.4)).toEqual({ isSnapped: true, seconds: 30 });
    expect(snap(20)).toEqual({ isSnapped: false, seconds: 20 });
  });

  it("settles on the nearest edge where two are in reach", () => {
    const snap = edgeSnap({
      scale,
      targets: [...targets, { end: 31, start: 31 }],
      width: WIDTH,
    });
    expect(snap(30.6)).toEqual({ isSnapped: true, seconds: 31 });
  });

  it("leaves out the edges of the item dragged", () => {
    const snap = edgeSnap({ except: "scene", scale, targets, width: WIDTH });
    expect(snap(40.2)).toEqual({ isSnapped: false, seconds: 40.2 });
    expect(snap(30.2)).toEqual({ isSnapped: true, seconds: 30 });
  });

  it("measures each end where it is drawn, as the end of what stops there", () => {
    // Reordered: 0 to 50 s plays after 50 to 100 s, so 50 s is where the
    // strip starts as a start and where it ends as an end.
    const map: TimeMap = {
      duration: 100,
      toPlayed: (recorded, isEnd) =>
        recorded > 50 || (recorded === 50 && !isEnd)
          ? recorded - 50
          : recorded + 50,
      toRecorded: (played) => (played < 50 ? played + 50 : played - 50),
    };
    const snap = edgeSnap({
      scale: timelineScale(map),
      targets: stripSnapTargets({
        annotations: [{ end: 50, id: "arrow", start: 45 }],
      }),
      width: WIDTH,
    });
    // Near the strip's end, 49.6 s recorded plays at 99.6 s.
    expect(snap(49.6)).toEqual({ isSnapped: true, seconds: 50 });
    // Near its start, 50.4 s recorded plays at 0.4 s: no end is drawn there.
    expect(snap(50.4)).toEqual({ isSnapped: false, seconds: 50.4 });
  });
});
