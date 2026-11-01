// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  linesOf,
  type RecordingBeats,
  recordingBeats,
  type RecordingReadings,
  withoutScreen,
} from "./recording-beats";

const empty: RecordingReadings = {
  clicks: [],
  duration: 20,
  moments: [],
  picture: [],
  shortcuts: [],
  typing: [],
  words: [],
};

const word = (start: number, end: number, text: string) => ({
  end,
  start,
  text,
});

/** Beats and lulls tile the recording, in order, with no gap or overlap. */
function expectTiled({ beats, lulls }: RecordingBeats, duration: number) {
  const spans = [...beats, ...lulls]
    .map(({ span }) => span)
    .sort((one, other) => one.start - other.start);
  expect(spans[0].start).toBe(0);
  expect(spans[spans.length - 1].end).toBe(duration);
  spans.slice(1).forEach((span, index) => {
    expect(span.start).toBeCloseTo(spans[index].end, 9);
  });
}

describe("linesOf", () => {
  it("ends a line at a long pause or a sentence's end", () => {
    expect(
      linesOf([
        word(1, 1.3, " Hey"),
        word(1.4, 1.8, "there."),
        word(1.9, 2.2, "Now"),
        word(2.3, 2.6, "this"),
        word(4, 4.4, "later"),
      ]),
    ).toMatchObject([
      { end: 1.8, start: 1, text: "Hey there." },
      { end: 2.6, start: 1.9, text: "Now this" },
      { end: 4.4, start: 4, text: "later" },
    ]);
  });
});

describe("recordingBeats", () => {
  it("is one lull for a recording where nothing happens", () => {
    expect(recordingBeats(empty)).toEqual({
      beats: [],
      lulls: [{ id: "lull-0", span: { end: 20, start: 0 } }],
    });
  });

  it("finds the beats of a recording with a moment, clicks, a shortcut and typing", () => {
    // The test recording: a moment flagged at once, a click, ⌥↩, typing,
    // and a last click.
    const found = recordingBeats({
      ...empty,
      clicks: [
        { at: 8.617, count: 1 },
        { at: 13.905, count: 1 },
      ],
      duration: 15.041,
      moments: [{ at: 0.59, color: "#ff383c", index: 0, name: "Custom" }],
      shortcuts: [{ at: 8.955, end: 10, id: "0", keys: ["Option", "Return"] }],
      typing: [9.295, 10.844, 11.348],
    });
    expect(found.beats.map(({ beat }) => beat.kind)).toEqual([
      "moment",
      "click",
      "keys",
      "type",
      "click",
    ]);
    expect(found.beats[0].span.start).toBe(0);
    expect(found.beats[1].span.start).toBeCloseTo(7.617, 9);
    expect(found.beats[1].span.end).toBeCloseTo(8.555, 9);
    expectTiled(found, 15.041);
  });

  it("lets the later beat start after one it would leave too short", () => {
    const { beats } = recordingBeats({
      ...empty,
      clicks: [{ at: 5, count: 1 }],
      shortcuts: [{ at: 4.3, end: 5, id: "1", keys: ["Command", "S"] }],
    });
    expect(beats.map(({ span }) => span)).toEqual([
      { end: 4.8, start: 3.9 },
      { end: 5.5, start: 4.8 },
    ]);
  });

  it("gives a gap too short to be a lull to the beat before it", () => {
    const found = recordingBeats({
      ...empty,
      duration: 10,
      words: [word(1, 2, "One."), word(2.1, 3, "Two.")],
    });
    expect(found.beats.map(({ span }) => span)).toEqual([
      { end: 2.1, start: 1 },
      { end: 3, start: 2.1 },
    ]);
    expectTiled(found, 10);
  });

  it("makes new pictures beats and keeps where a lull's picture moves", () => {
    const found = recordingBeats({
      ...empty,
      duration: 12,
      picture: [
        { end: 4.5, start: 4, state: "new" },
        { end: 9, start: 7, state: "moving" },
      ],
    });
    expect(found.beats).toEqual([
      {
        beat: { kind: "screen" },
        id: "screen-4000",
        span: { end: 5, start: 4 },
      },
    ]);
    expect(found.lulls[1]).toEqual({
      id: "lull-5000",
      moving: [{ end: 9, start: 7 }],
      span: { end: 12, start: 5 },
    });
  });
});

describe("withoutScreen", () => {
  // Said, done on screen, said again, then a moment flagged.
  const readings: RecordingReadings = {
    ...empty,
    clicks: [{ at: 6, count: 1 }],
    duration: 30,
    moments: [{ at: 26, color: "#ff383c", index: 0, name: "Win" }],
    picture: [
      { end: 15, start: 14, state: "new" },
      { end: 18, start: 16, state: "moving" },
    ],
    shortcuts: [{ at: 12, end: 13, id: "0", keys: ["Command", "S"] }],
    typing: [8, 9],
    words: [word(1, 3, "First line."), word(20, 22, "Second.")],
  };

  it("keeps every beat with the screen in", () => {
    const found = recordingBeats(readings);
    expect(found.beats.map(({ beat }) => beat.kind)).toEqual([
      "speech",
      "click",
      "type",
      "keys",
      "screen",
      "speech",
      "moment",
    ]);
    expect(found.lulls.some(({ moving }) => moving)).toBe(true);
    expectTiled(found, 30);
  });

  it("keeps the voice and the moments, and lulls the gaps between them", () => {
    const found = recordingBeats(withoutScreen(readings));
    expect(
      found.beats.map(({ beat, id, span }) => ({ id, kind: beat.kind, span })),
    ).toEqual([
      { id: "speech-1000", kind: "speech", span: { end: 3, start: 1 } },
      { id: "speech-20000", kind: "speech", span: { end: 22, start: 20 } },
      { id: "moment-0", kind: "moment", span: { end: 28, start: 22 } },
    ]);
    // Nothing on the screen moves any more, so no lull holds motion.
    expect(found.lulls).toEqual([
      { id: "lull-0", span: { end: 1, start: 0 } },
      { id: "lull-3000", span: { end: 20, start: 3 } },
      { id: "lull-28000", span: { end: 30, start: 28 } },
    ]);
    expectTiled(found, 30);
  });
});
