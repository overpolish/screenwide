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
import { takenOut } from "../strip/taken-out";

import { type RecordingLull } from "./recording-beats";
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
import {
  lullPlayed,
  recordingStripLulls,
  squeezedTo,
} from "./recording-strip-lulls";
import { recordingStripTime } from "./recording-strip-time";

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

describe("lulls", () => {
  const found: RecordingLull[] = [
    { id: "still", span: { end: 4, start: 0 } },
    {
      id: "moving",
      moving: [{ end: 9, start: 7 }],
      span: { end: 10, start: 6 },
    },
  ];
  const lullsOf = (edit: typeof whole) =>
    recordingStripLulls({
      cuts: timeOf(edit).cuts,
      edit,
      found,
      recorded: RECORDED,
    });

  it("squeezes every lull by one share of its give, and back", () => {
    const before = lullsOf(whole);
    // Four seconds give three at four times; four moving seconds give two.
    expect(before.maxSqueeze).toBe(5);
    const squeezed = squeezedTo(whole, before, {
      recorded: RECORDED,
      squeeze: 2.5,
    });
    const after = lullsOf(squeezed);
    expect(after.share).toBeCloseTo(0.5, 2);
    expect(after.squeeze).toBeCloseTo(2.5, 2);
    expect(after.lulls.map(({ play }) => play.kind)).toEqual([
      "squeeze",
      "squeeze",
    ]);
    expect(
      squeezedTo(squeezed, after, { recorded: RECORDED, squeeze: 0 }).segments,
    ).toHaveLength(1);
  });

  it("keeps a lull as recorded while the rest squeeze", () => {
    const squeezed = squeezedTo(whole, lullsOf(whole), {
      recorded: RECORDED,
      squeeze: 2.5,
    });
    const kept = lullPlayed(squeezed, lullsOf(squeezed), {
      id: "still",
      play: { kind: "keep" },
      recorded: RECORDED,
    });
    expect(lullsOf(kept).lulls.map(({ play }) => play.kind)).toEqual([
      "keep",
      "squeeze",
    ]);
  });
});

describe("what is taken out", () => {
  const found: RecordingLull[] = [
    { id: "still", span: { end: 4, start: 0 } },
    { id: "late", span: { end: 10, start: 6 } },
  ];
  const takenOf = (edit: typeof whole) => {
    const time = timeOf(edit);
    return takenOut(time.cuts, time.map.duration, RECORDED);
  };
  const squeezeBy = (edit: typeof whole, squeeze: number) =>
    squeezedTo(
      edit,
      recordingStripLulls({
        cuts: timeOf(edit).cuts,
        edit,
        found,
        recorded: RECORDED,
      }),
      { recorded: RECORDED, squeeze },
    );

  it("takes nothing out of an unedited recording", () => {
    expect(takenOf(whole)).toEqual({ deleted: 0, squeezed: 0 });
  });

  it("counts what a cut leaves out as deleted", () => {
    const taken = takenOf(cutStretch(whole, { end: 6, start: 4 }, RECORDED));
    expect(taken.deleted).toBeCloseTo(2, 9);
    expect(taken.squeezed).toBeCloseTo(0, 9);
  });

  it("counts what the squeeze saves as squeezed", () => {
    const taken = takenOf(squeezeBy(whole, 3));
    expect(taken.deleted).toBe(0);
    expect(taken.squeezed).toBeCloseTo(3, 2);
  });

  it("splits what a cut and a squeeze take out between them", () => {
    const cut = cutStretch(whole, { end: 6, start: 4 }, RECORDED);
    const both = squeezeBy(cut, 3);
    const taken = takenOf(both);
    expect(taken.deleted).toBeCloseTo(2, 9);
    expect(taken.squeezed).toBeCloseTo(3, 2);
    expect(taken.deleted + taken.squeezed).toBeCloseTo(
      RECORDED - timeOf(both).map.duration,
      9,
    );
  });

  it("counts overlapping cuts once", () => {
    expect(
      takenOut(
        [
          { end: 5, start: 2 },
          { end: 6, start: 4 },
        ],
        6,
        RECORDED,
      ),
    ).toEqual({ deleted: 4, squeezed: 0 });
  });
});
