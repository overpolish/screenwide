// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { createPlayhead } from "../scrub-playhead";
import { recordedTime, type TimeMap } from "../timeline-scale";

import {
  recordingStripLanes,
  type RecordingStripLaneSource,
} from "./recording-strip-lanes";
import { recordingStripTime } from "./recording-strip-time";

import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";

const RECORDED = 10;

/** The lanes over `clips` with the playhead at `atSeconds`, the moments
 * they asked the playhead to go to, and the scenes they changed to. */
function lanesAt(
  atSeconds: number,
  clips: RecordingSceneClip[] = [],
  time: { map: TimeMap; runs: { end: number; start: number }[] } = {
    map: recordedTime(RECORDED),
    runs: [{ end: RECORDED, start: 0 }],
  },
) {
  const playhead = createPlayhead();
  playhead.publish(atSeconds, atSeconds / RECORDED);
  const seeks: number[] = [];
  const changed: RecordingSceneClip[][] = [];
  const source: RecordingStripLaneSource = {
    annotations: {
      clearSelection: () => undefined,
      clips: [],
      onClipsChange: () => undefined,
      onSelect: () => undefined,
      selectedIds: new Set(),
    },
    changeCanvasTool: () => undefined,
    map: time.map,
    openCaptions: () => undefined,
    playhead,
    runs: time.runs,
    scenes: {
      clips,
      drawingOf: () => ({ aspect: 16 / 9 }),
      isAvailable: true,
      onClipsChange: (next) => {
        changed.push(next);
      },
      selection: {
        ids: new Set(),
        onClear: () => undefined,
        onSelect: () => undefined,
      },
    },
    seek: (seconds) => {
      seeks.push(seconds);
    },
    steps: {
      once: () => undefined,
      step: (next) => {
        if (typeof next === "function") next();
      },
    },
  };
  return { changed, lanes: recordingStripLanes(source).lanes, seeks };
}

describe("a scene drawn on the rail", () => {
  it("brings the playhead to it, once arrived, where it stood before it", () => {
    const { lanes, seeks } = lanesAt(1, [
      { endMs: 3000, id: "before", preset: "full", startMs: 500 },
    ]);
    lanes.onSceneDraw(4, 6);
    expect(seeks).toEqual([4.6]);
  });

  it("brings the playhead to it, once arrived, where it stood after it", () => {
    const { lanes, seeks } = lanesAt(8);
    lanes.onSceneDraw(6, 4);
    expect(seeks).toEqual([4.6]);
  });

  it("leaves the playhead where it stands inside it", () => {
    const { lanes, seeks } = lanesAt(5);
    lanes.onSceneDraw(4, 6);
    expect(seeks).toEqual([]);
  });

  it("grows a drag too short to the shortest drawn scene, about its middle", () => {
    const { changed, lanes } = lanesAt(0);
    lanes.onSceneDraw(4, 4.2);
    expect(changed).toMatchObject([[{ endMs: 4600, startMs: 3600 }]]);
  });

  it("slides a grown scene clear of the neighbour it would run into", () => {
    const { changed, lanes } = lanesAt(0, [
      { endMs: 4000, id: "before", preset: "full", startMs: 1000 },
    ]);
    lanes.onSceneDraw(4.1, 4.2);
    expect(changed).toMatchObject([
      [{ id: "before" }, { endMs: 5000, startMs: 4000 }],
    ]);
  });
});

// Ten recorded seconds played with 4 to 6 moved to the end: 0 to 4, then
// 6 to 10, then 4 to 6.
const MOVED: TimeMap = {
  duration: RECORDED,
  toPlayed: (recorded) =>
    recorded < 4 ? recorded : recorded < 6 ? recorded + 4 : recorded - 2,
  toRecorded: (played) =>
    played < 4 ? played : played < 8 ? played + 2 : played - 4,
};
const MOVED_RUNS = [
  { end: 4, start: 0 },
  { end: 8, start: 4 },
  { end: 10, start: 8 },
];

describe("a scene played across a jump", () => {
  const across = () =>
    lanesAt(0, [{ endMs: 5000, id: "across", preset: "full", startMs: 3000 }], {
      map: MOVED,
      runs: MOVED_RUNS,
    });

  it("is drawn where it plays, each side of the jump", () => {
    const spans = across().lanes.scenes.map(({ end, id, start }) => ({
      end: Number(end.toFixed(3)),
      id,
      start: Number(start.toFixed(3)),
    }));
    // Not 3 to 7, which takes in 4 to 6, played at the end.
    expect(spans).toEqual([
      { end: 4, id: "across", start: 3 },
      { end: 7, id: "across:1", start: 6 },
    ]);
  });

  it("is chosen and retimed by any of its parts, at its own ends", () => {
    const { changed, lanes } = across();
    const [, after] = lanes.scenes;
    lanes.onSceneRetime(after.id, after.start, 7.5);
    expect(changed).toMatchObject([[{ endMs: 5500, startMs: 3000 }]]);
  });

  it("keeps its span where a part's end at the jump is pulled", () => {
    const { changed, lanes } = across();
    const [, after] = lanes.scenes;
    lanes.onSceneRetime(after.id, 6.5, after.end);
    expect(changed).toMatchObject([[{ endMs: 5000, startMs: 3000 }]]);
  });

  it("is drawn where it plays where a run starts a hair early", () => {
    // 0.94 to 2.16 of 8.986 s moved before 7.395: the last run's start
    // read as in the run before it, and its part as reaching back there.
    const recorded = 8.986;
    const segments = [
      [0, 0.9404206783251935],
      [2.1598888078070666, 7.395084456311113],
      [0.9404206783251935, 2.1598888078070666],
      [7.395084456311113, recorded],
    ].map(([start, end], id) => ({
      id,
      sourceEnd: end / recorded,
      sourceStart: start / recorded,
    }));
    const time = recordingStripTime(
      { artifactId: 1, nextSegmentId: 4, segments },
      recorded,
    );
    const { lanes } = lanesAt(
      0,
      [{ endMs: 7500, id: "late", preset: "full", startMs: 6000 }],
      time,
    );
    expect(
      lanes.scenes.map(({ end, start }) => [
        Number(start.toFixed(3)),
        Number(end.toFixed(3)),
      ]),
    ).toEqual([
      [7.219, 7.395],
      [0.94, 2.16],
      [7.395, 7.5],
    ]);
  });
});
