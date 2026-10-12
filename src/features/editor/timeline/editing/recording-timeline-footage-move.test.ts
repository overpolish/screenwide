// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
  setRecordingTimelineSegmentPlaybackRate,
} from "./recording-timeline-edit";
import { moveRecordingTimelineFootage } from "./recording-timeline-footage-move";

const DURATION_MS = 10_000;

/** `edit` with the footage from `startMs` to `endMs` moved before
 * `beforeMs`. */
const move = (
  edit: RecordingTimelineEdit,
  [startMs, endMs]: [number, number],
  beforeMs: number | null,
) =>
  moveRecordingTimelineFootage(edit, { beforeMs, endMs, startMs }, DURATION_MS);

/** The footage each segment plays, in playback order, in milliseconds. */
const played = (edit: RecordingTimelineEdit) =>
  edit.segments.map((segment) => [
    Math.round(segment.sourceStart * DURATION_MS),
    Math.round(segment.sourceEnd * DURATION_MS),
  ]);

const whole = createRecordingTimelineEdit(1);

describe("moveRecordingTimelineFootage", () => {
  it("moves footage later, closing the gap it leaves", () => {
    expect(played(move(whole, [2_000, 3_000], 6_000))).toEqual([
      [0, 2_000],
      [3_000, 6_000],
      [2_000, 3_000],
      [6_000, 10_000],
    ]);
  });

  it("moves footage earlier", () => {
    expect(played(move(whole, [6_000, 7_000], 2_000))).toEqual([
      [0, 2_000],
      [6_000, 7_000],
      [2_000, 6_000],
      [7_000, 10_000],
    ]);
  });

  it("moves footage to the end of the timeline", () => {
    expect(played(move(whole, [2_000, 3_000], null))).toEqual([
      [0, 2_000],
      [3_000, 10_000],
      [2_000, 3_000],
    ]);
  });

  it("changes nothing for a drop onto the footage or either of its ends", () => {
    expect(move(whole, [2_000, 3_000], 2_500)).toBe(whole);
    expect(move(whole, [2_000, 3_000], 2_000)).toBe(whole);
    expect(move(whole, [2_000, 3_000], 3_000)).toBe(whole);
  });

  it("moves what plays between the ends of footage across a jump", () => {
    // 6-10 s plays first, then 0-6 s.
    const reordered = move(whole, [6_000, 10_000], 0);
    // From 9 s, played first, to 1 s, played after the jump.
    expect(played(move(reordered, [9_000, 1_000], null))).toEqual([
      [6_000, 9_000],
      [1_000, 6_000],
      [9_000, 10_000],
      [0, 1_000],
    ]);
  });

  it("lands footage dropped before a cut moment after the footage before it", () => {
    // 4-5 s cut away: footage dropped before 4.5 s lands after 0-4 s.
    const cut = {
      ...whole,
      nextSegmentId: 2,
      segments: [
        { id: 0, sourceEnd: 0.4, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.5 },
      ],
    };
    expect(played(move(cut, [8_000, 9_000], 4_500))).toEqual([
      [0, 4_000],
      [8_000, 9_000],
      [5_000, 8_000],
      [9_000, 10_000],
    ]);
  });

  it("keeps the speed of the footage it moves", () => {
    const doubled = setRecordingTimelineSegmentPlaybackRate(whole, 0, 2);
    const moved = move(doubled, [2_000, 3_000], 6_000);
    expect(moved.segments.map((segment) => segment.playbackRate)).toEqual([
      2, 2, 2, 2,
    ]);
  });

  it("takes an existing edge a fraction of a millisecond away as the cut", () => {
    const cut = {
      ...whole,
      nextSegmentId: 2,
      segments: [
        { id: 0, sourceEnd: 0.20003, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.20003 },
      ],
    };
    // 2 000 ms lies 0.3 ms inside the first segment: no sliver is left.
    expect(move(cut, [2_000, 10_000], 0).segments.map(({ id }) => id)).toEqual([
      1, 0,
    ]);
  });
});
