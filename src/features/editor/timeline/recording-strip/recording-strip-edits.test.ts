// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  type RecordingOverlayTimelines,
  retimeRecordingOverlays,
} from "../../recording/recording-overlay-retime";
import {
  createRecordingTimelineEdit,
  type RecordingTimelineEdit,
} from "../editing/recording-timeline-edit";
import { keptOf } from "../strip/strip-beat-trim";

import {
  cutStretch,
  moved,
  putBack,
  restoredCuts,
  splitAt,
  trimmed,
  unsplitAt,
  withListsOf,
} from "./recording-strip-edits";
import { recordingStripTime } from "./recording-strip-time";
import { chosenSpans, partSpans } from "./use-recording-strip-edits";

const RECORDED = 10;
const whole = createRecordingTimelineEdit(1);
const timeOf = (edit: typeof whole) => recordingStripTime(edit, RECORDED);
const spansOf = (edit: typeof whole) =>
  edit.segments.map(({ sourceEnd, sourceStart }) => [
    sourceStart * RECORDED,
    sourceEnd * RECORDED,
  ]);

describe("a drag of a beat's end", () => {
  // A scene that ends where play jumps, on a recording whose length puts
  // the jump between whole ms.
  const recorded = 10.105;
  const start: RecordingTimelineEdit = {
    ...moved(
      whole,
      { before: null, spans: [{ end: 7.136, start: 6.592 }] },
      recorded,
    ),
    sceneClips: [
      { endMs: 6592, id: "scene", preset: "camera-only", startMs: 3789 },
    ],
  };
  const beat = { end: 4.288, start: 3 };

  it("leaves what ends where play jumps ending there", () => {
    const timelines: RecordingOverlayTimelines = new WeakMap();
    let edit = start;
    // Trimmed in step by step, each made from the one before.
    for (let step = 1; step <= 40; step += 1) {
      const next = trimmed(
        edit,
        {
          at: beat.end - (0.145 * step) / 40,
          edge: "end",
          kept: keptOf(beat, recordingStripTime(edit, recorded).cuts),
          span: beat,
        },
        recorded,
      );
      edit = retimeRecordingOverlays(
        { next: withListsOf(start, next), previous: edit },
        { sourceDurationMs: recorded * 1000, timelines },
      );
    }
    const [, jump] = recordingStripTime(edit, recorded).runs;
    expect(edit.sceneClips?.[0].endMs).toBe(Math.round(jump.start * 1000));
  });
});

describe("strip edits", () => {
  it("puts a cut back from its seam", () => {
    const cut = cutStretch(whole, { end: 6, start: 2 }, RECORDED);
    expect(spansOf(restoredCuts(cut, ["cut-200000"]))).toEqual([[0, 10]]);
  });

  it("trims a beat's start and brings it back", () => {
    const beat = { end: 5, start: 2 };
    const trim = trimmed(
      whole,
      { at: 3, edge: "start", kept: beat, span: beat },
      RECORDED,
    );
    expect(timeOf(trim).cuts).toMatchObject([{ end: 3, start: 2 }]);
    const back = trimmed(
      trim,
      { at: 2, edge: "start", kept: { end: 5, start: 3 }, span: beat },
      RECORDED,
    );
    expect(spansOf(back)).toEqual([[0, 10]]);
  });

  it("splits and takes the split away", () => {
    const split = splitAt(whole, 4, RECORDED);
    expect(timeOf(split).splits).toEqual([4]);
    expect(unsplitAt(split, 4, RECORDED).segments).toHaveLength(1);
  });

  it("cuts the piece of a beat between two splits, and only that", () => {
    const split = splitAt(splitAt(whole, 4, RECORDED), 6, RECORDED);
    const spanOf = partSpans(
      { beats: [{ id: "b", span: { end: 8, start: 2 } }], lulls: [] },
      timeOf(split),
    );
    const piece = chosenSpans({ id: "b:1", kind: "beat" }, spanOf);
    expect(piece).toEqual([{ end: 6, start: 4 }]);
    expect(spansOf(cutStretch(split, piece[0], RECORDED))).toEqual([
      [0, 4],
      [6, 10],
    ]);
  });

  it("moves a stretch and puts it back from its jump", () => {
    const move = moved(
      whole,
      { before: 8, spans: [{ end: 4, start: 2 }] },
      RECORDED,
    );
    expect(spansOf(move)).toEqual([
      [0, 2],
      [4, 8],
      [2, 4],
      [8, 10],
    ]);
    expect(timeOf(move).jumps).toEqual([4, 2, 8]);
    expect(timeOf(move).splits).toEqual([]);
    for (const at of [4, 2, 8])
      expect(spansOf(putBack(move, at, RECORDED))).toEqual([[0, 10]]);
  });
});
