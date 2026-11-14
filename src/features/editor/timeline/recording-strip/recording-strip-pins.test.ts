// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  timelineAnnotationPin,
  withAnnotationPins,
} from "./recording-strip-pins";

import type { RecordingAnnotationPin } from "../../recording/annotations/recording-annotations";
import type { RecordingPinStatus } from "../../recording/annotations/use-recording-pin-status";

const status = (covered: [number, number][]): RecordingPinStatus => ({
  covered,
  hidden: [],
  progress: null,
  under: [],
  weak: [],
});

// Pinned at 2 s, put right by hand at 4 s, and said out of view from 6 s
// with nothing after.
const pin: RecordingAnnotationPin = {
  keyframes: [
    { dx: 0, dy: 0, ms: 2_000 },
    { dx: 3, dy: 1, ms: 4_000 },
    { dx: 0, dy: 0, ms: 6_000, outOfView: true },
  ],
  pinnedMs: 2_000,
};

/** A recording played as it was recorded. */
const asRecorded = (seconds: number) => seconds;
/** A bar over `start` to `end` of a recording played as recorded. */
const within = (start: number, end: number) => ({
  part: { end, start },
  toPlayed: asRecorded,
});
const WHOLE = within(1, 10);

describe("a pinned annotation's bar", () => {
  it("ticks every keyframe put by hand, in seconds, but not the pinned one", () => {
    expect(timelineAnnotationPin(pin, undefined, WHOLE).corrections).toEqual([
      4, 6,
    ]);
  });

  it("hatches where it was said out of view on past its part's end", () => {
    expect(timelineAnnotationPin(pin, undefined, WHOLE).outOfView).toEqual([
      { end: 10, start: 6 },
    ]);
  });

  it("joins the tracker's hidden stretches with the hand's where they overlap", () => {
    expect(
      timelineAnnotationPin(
        pin,
        status([
          [2_500, 3_000],
          [5_000, 7_000],
        ]),
        WHOLE,
      ).outOfView,
    ).toEqual([
      { end: 3, start: 2.5 },
      { end: 10, start: 5 },
    ]);
  });

  it("keeps each part of a bar split at a jump to its own span", () => {
    const part = timelineAnnotationPin(
      pin,
      status([[1_000, 3_000]]),
      within(2.5, 5),
    );
    expect(part).toEqual({
      corrections: [4],
      outOfView: [{ end: 3, start: 2.5 }],
      unsure: [],
    });
  });

  it("marks where the tracker lost the content, found it off the frame or under a cover, joined and kept to its part", () => {
    const unsure = timelineAnnotationPin(
      pin,
      {
        ...status([[5_000, 6_000]]),
        hidden: [[7_000, 8_000]],
        under: [[7_500, 9_000]],
        weak: [
          [500, 1_500],
          [3_000, 3_500],
        ],
      },
      within(1, 8.5),
    ).unsure;
    expect(unsure).toEqual([
      { end: 1.5, start: 1 },
      { end: 3.5, start: 3 },
      { end: 8.5, start: 7 },
    ]);
  });

  it("is sure everywhere before the tracker has said", () => {
    expect(timelineAnnotationPin(pin, undefined, WHOLE).unsure).toEqual([]);
  });

  it("lays its stretches out where they play, past a gap before them", () => {
    // 3 s of gap inserted at 5 s: from there on, everything plays 3 s on.
    const pastGap = (seconds: number) => (seconds < 5 ? seconds : seconds + 3);
    const shown = timelineAnnotationPin(pin, undefined, {
      part: { end: 13, start: 1 },
      toPlayed: pastGap,
    });
    expect(shown).toEqual({
      corrections: [4, 9],
      outOfView: [{ end: 13, start: 9 }],
      unsure: [],
    });
  });
});

describe("the strip's annotation bars", () => {
  it("give each part of a pinned clip's bar its pin, and leave the rest", () => {
    const bars = withAnnotationPins(
      [
        { end: 3, id: "pinned", kind: "arrow", lane: 0, start: 1 },
        { end: 9, id: "pinned:1", kind: "arrow", lane: 0, start: 5 },
        { end: 9, id: "free", kind: "counter", lane: 1, start: 1 },
      ],
      {
        clips: [
          { annotation: { id: "pinned" }, pin },
          { annotation: { id: "free" } },
        ],
        statuses: new Map(),
      },
      asRecorded,
    );
    expect(bars.map((bar) => bar.pin)).toEqual([
      { corrections: [], outOfView: [], unsure: [] },
      { corrections: [6], outOfView: [{ end: 9, start: 6 }], unsure: [] },
      undefined,
    ]);
  });
});
