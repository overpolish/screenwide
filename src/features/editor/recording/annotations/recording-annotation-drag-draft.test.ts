// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { createRecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { TIMED_LANE_ROW_HEIGHT_PX } from "../../timeline/tracks/timed-lane-layout";

import { carriedClips } from "./recording-annotation-drag-draft";
import { RecordingAnnotationClip } from "./recording-annotations";

const clip = (
  id: string,
  startMs: number,
  endMs: number,
): RecordingAnnotationClip => ({
  annotation: {
    animated: true,
    id,
    shape: {
      control: { x: 5, y: 0 },
      end: { x: 10, y: 0 },
      kind: "arrow",
      start: { x: 0, y: 0 },
    },
    style: {
      align: "left",
      blur: false,
      color: "#ffcc00",
      handDrawn: false,
      head: "end",
      manual: false,
      radius: 0,
      redaction: "erase",
      shadow: false,
      softness: 0,
      strength: 0,
      tint: false,
      width: 8,
    },
  },
  endMs,
  startMs,
  trackId: "primary",
});

const carry = (
  original: RecordingAnnotationClip[],
  ids: string[],
  { deltaOutput = 0, rows = 0 } = {},
) =>
  carriedClips({
    deltaOutput,
    edit: createRecordingTimelineEdit(1),
    ids: new Set(ids),
    lift: rows * TIMED_LANE_ROW_HEIGHT_PX,
    original,
    sourceDurationMs: 100_000,
  });

const order = (clips: RecordingAnnotationClip[]) =>
  clips.map((item) => item.annotation.id);

describe("carriedClips", () => {
  it("moves a group together and stops it as one at the timeline's end", () => {
    const moved = carry(
      [
        clip("a", 10_000, 20_000),
        clip("x", 30_000, 40_000),
        clip("b", 50_000, 90_000),
      ],
      ["a", "b"],
      { deltaOutput: 0.3 },
    );
    expect(moved.map(({ endMs, startMs }) => [startMs, endMs])).toEqual([
      [20_000, 30_000],
      [30_000, 40_000],
      [60_000, 100_000],
    ]);
  });

  it("lifts a group over what it overlaps without reordering the group", () => {
    const lifted = carry(
      [clip("a", 0, 10_000), clip("x", 0, 10_000), clip("b", 0, 10_000)],
      ["a", "b"],
      { rows: 1 },
    );
    expect(order(lifted)).toEqual(["x", "a", "b"]);
  });

  it("lowers a group under what it overlaps without reordering the group", () => {
    const lowered = carry(
      [clip("x", 0, 10_000), clip("a", 0, 10_000), clip("b", 0, 10_000)],
      ["a", "b"],
      { rows: -1 },
    );
    expect(order(lowered)).toEqual(["a", "b", "x"]);
  });

  it("holds a member back behind another it cannot pass", () => {
    // `a` meets nothing below it, so `b` above it has only `a` to pass.
    const lowered = carry(
      [
        clip("x", 20_000, 30_000),
        clip("a", 0, 10_000),
        clip("b", 5_000, 25_000),
      ],
      ["a", "b"],
      { rows: -1 },
    );
    expect(order(lowered)).toEqual(["x", "a", "b"]);
  });
});
