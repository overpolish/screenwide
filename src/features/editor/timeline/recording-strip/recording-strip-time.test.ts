// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { createRecordingTimelineEdit } from "../editing/recording-timeline-edit";

import { cutStretch } from "./recording-strip-edits";
import {
  isPlayed,
  jumpIndexes,
  recordingStripTime,
} from "./recording-strip-time";

const RECORDED = 10;
const whole = createRecordingTimelineEdit(1);
const timeOf = (edit: typeof whole) => recordingStripTime(edit, RECORDED);

describe("recordingStripTime", () => {
  it("plays an unedited recording as it was recorded", () => {
    const time = timeOf(whole);
    expect(time.map.duration).toBe(10);
    expect(time.map.toPlayed(4)).toBe(4);
    expect(time.map.toRecorded(4)).toBe(4);
    expect(time).toMatchObject({ cuts: [], jumps: [], splits: [] });
  });

  it("plays a cut for no time, where its seam is", () => {
    const time = timeOf(cutStretch(whole, { end: 6, start: 2 }, RECORDED));
    expect(time.cuts).toEqual([{ end: 6, id: "cut-200000", start: 2 }]);
    expect(time.map.duration).toBeCloseTo(6, 9);
    expect(time.map.toPlayed(4)).toBeCloseTo(2, 9);
    expect(time.map.toPlayed(7)).toBeCloseTo(3, 9);
    expect(time.map.toRecorded(3)).toBeCloseTo(7, 9);
  });

  it("finds what a cut took out whole plays for no time", () => {
    const { map } = timeOf(cutStretch(whole, { end: 6, start: 2 }, RECORDED));
    expect(isPlayed(map, { end: 5, start: 3 })).toBe(false);
    expect(isPlayed(map, { end: 6, start: 2 })).toBe(false);
    expect(isPlayed(map, { end: 7, start: 5 })).toBe(true);
    expect(isPlayed(map, { end: 2.5, start: 1 })).toBe(true);
  });

  it("reads each run's start as the start of what plays there", () => {
    // Each stretch's end is summed from its length, a hair either side of
    // where the next starts: these runs' starts fell between two stretches,
    // read as the end of the recording, or in the stretch before.
    const recorded = 8.986;
    const segmentsOf = (spans: number[][]) =>
      spans.map(([start, end], id) => ({
        id,
        sourceEnd: end / recorded,
        sourceStart: start / recorded,
      }));
    for (const segments of [
      segmentsOf([
        [0, 2.674589292032919],
        [3.5315292424585345, 7.888378103440805],
        [8.247000908869786, 8.422233624862615],
        [7.888378103440805, 8.247000908869786],
      ]),
      segmentsOf([
        [0, 0.9404206783251935],
        [2.1598888078070666, 7.395084456311113],
        [0.9404206783251935, 2.1598888078070666],
        [7.395084456311113, recorded],
      ]),
    ]) {
      const { map, runs } = recordingStripTime(
        { ...whole, segments },
        recorded,
      );
      expect(
        runs.slice(1).map(({ start }) => map.toRecorded(start) / recorded),
      ).toEqual(
        jumpIndexes(segments).map((index) => segments[index].sourceStart),
      );
    }
  });
});
