// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { createRecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { takenOut } from "../strip/taken-out";

import { type RecordingLull } from "./recording-beats";
import { cutStretch } from "./recording-strip-edits";
import {
  lullPlayed,
  recordingStripLulls,
  squeezedTo,
} from "./recording-strip-lulls";
import { atRate, rateAt } from "./recording-strip-source";
import { recordingStripTime } from "./recording-strip-time";

const RECORDED = 10;
const whole = createRecordingTimelineEdit(1);
const timeOf = (edit: typeof whole) => recordingStripTime(edit, RECORDED);

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

  it("leaves a retimed beat's speed alone as the lulls squeeze", () => {
    // The beat between the two lulls, played twice as fast.
    const retimed = atRate(whole, { end: 0.6, rate: 2, start: 0.4 });
    const squeezed = squeezedTo(retimed, lullsOf(retimed), {
      recorded: RECORDED,
      squeeze: 2.5,
    });
    const loosened = squeezedTo(squeezed, lullsOf(squeezed), {
      recorded: RECORDED,
      squeeze: 0,
    });
    for (const edit of [squeezed, loosened])
      expect(rateAt(edit, 5, RECORDED)).toBe(2);
    expect(lullsOf(squeezed).lulls.map(({ play }) => play.kind)).toEqual([
      "squeeze",
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

  it("counts what a beat played faster saves as squeezed", () => {
    const taken = takenOf(atRate(whole, { end: 0.6, rate: 2, start: 0.4 }));
    expect(taken.deleted).toBe(0);
    expect(taken.squeezed).toBeCloseTo(1, 9);
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
