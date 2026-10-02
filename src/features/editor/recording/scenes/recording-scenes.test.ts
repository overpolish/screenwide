// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  createRecordingTimelineEdit,
  cutRecordingTimeline,
  deleteRecordingTimelineSegment,
} from "../../timeline/editing/recording-timeline-edit";

import {
  insertRecordingSceneClip,
  moveRecordingSceneClip,
  RecordingSceneClip,
  resizeRecordingSceneClip,
} from "./recording-scenes";

const DURATION = 60_000;
const clip = (id: string, startMs: number, endMs: number) =>
  ({ endMs, id, preset: "split-two-thirds", startMs }) as RecordingSceneClip;
const insert = (clips: RecordingSceneClip[], atMs: number) =>
  insertRecordingSceneClip({
    atMs,
    clips,
    id: "new",
    preset: "split-two-thirds",
    sourceDurationMs: DURATION,
  });

describe("insertRecordingSceneClip", () => {
  it("runs five seconds from the playhead where there is room", () => {
    expect(insert([], 10_000)?.clip).toMatchObject({
      endMs: 15_000,
      startMs: 10_000,
    });
  });

  it("stops at the next clip and reaches back for the rest", () => {
    const next = insert(
      [clip("a", 0, 8_000), clip("b", 12_000, 20_000)],
      11_000,
    );
    expect(next?.clip).toMatchObject({ endMs: 12_000, startMs: 8_000 });
    expect(next?.clips.map((item) => item.id)).toEqual(["a", "new", "b"]);
  });

  it("reaches back from the end of the recording", () => {
    expect(insert([], 59_000)?.clip).toMatchObject({
      endMs: 60_000,
      startMs: 55_000,
    });
  });

  it("adds nothing inside a clip or where the gap is too short", () => {
    expect(insert([clip("a", 0, 8_000)], 4_000)).toBeNull();
    expect(
      insert([clip("a", 0, 8_000), clip("b", 8_200, 20_000)], 8_100),
    ).toBeNull();
  });
});

describe("moveRecordingSceneClip", () => {
  const edit = createRecordingTimelineEdit(1);

  const move = (clips: RecordingSceneClip[], id: string, deltaMs: number) =>
    moveRecordingSceneClip({
      clips,
      deltaOutput: deltaMs / DURATION,
      edit,
      id,
      sourceDurationMs: DURATION,
    });

  it("passes over a neighbour into the gap beyond, keeping its length", () => {
    const moved = move(
      [clip("a", 0, 5_000), clip("b", 10_000, 20_000)],
      "a",
      20_000,
    );
    expect(moved.map((item) => item.id)).toEqual(["b", "a"]);
    expect(moved[1]).toMatchObject({ endMs: 25_000, startMs: 20_000 });
  });

  it("fills a gap shorter than itself", () => {
    // b, 14s long, is dropped with its middle in the 2s between a and c.
    const moved = move(
      [
        clip("a", 0, 4_000),
        clip("c", 6_000, 10_000),
        clip("b", 20_000, 34_000),
      ],
      "b",
      -22_000,
    );
    expect(moved.map((item) => item.id)).toEqual(["a", "b", "c"]);
    expect(moved[1]).toMatchObject({ endMs: 6_000, startMs: 4_000 });
  });

  it("stops at the ends of the recording", () => {
    expect(move([clip("a", 10_000, 15_000)], "a", -60_000)[0]).toMatchObject({
      endMs: 5_000,
      startMs: 0,
    });
  });

  it("keeps how long it plays for across a cut", () => {
    // Cut 20s-30s out: the clip at 15s-20s slid 5s later plays across the cut.
    const cut = cutRecordingTimeline(
      cutRecordingTimeline(edit, 20_000 / DURATION),
      30_000 / DURATION,
    );
    const removed = deleteRecordingTimelineSegment(cut, cut.segments[1].id);
    const moved = moveRecordingSceneClip({
      clips: [clip("a", 15_000, 20_000)],
      deltaOutput: 2_500 / 50_000,
      edit: removed,
      id: "a",
      sourceDurationMs: DURATION,
    });
    expect(moved[0]).toMatchObject({ endMs: 32_500, startMs: 17_500 });
  });
});

describe("resizeRecordingSceneClip", () => {
  const edit = createRecordingTimelineEdit(1);
  const clips = [clip("a", 0, 5_000), clip("b", 10_000, 15_000)];

  it("trims no further than the neighbour", () => {
    const trimmed = resizeRecordingSceneClip({
      clips,
      edge: "startMs",
      edit,
      id: "b",
      output: 0,
      sourceDurationMs: DURATION,
    });
    expect(trimmed[1].startMs).toBe(5_000);
  });

  it("keeps the shortest a scene may be", () => {
    const trimmed = resizeRecordingSceneClip({
      clips,
      edge: "endMs",
      edit,
      id: "b",
      output: 0,
      sourceDurationMs: DURATION,
    });
    expect(trimmed[1]).toMatchObject({ endMs: 10_500, startMs: 10_000 });
  });
});
