// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  heardRuns,
  isHeardInOrder,
  movedFootage,
  transcriptInPlaybackOrder,
} from "./transcript-playback-order";
import { transcriptUnits } from "./transcript-units";
import { transcriptParagraphs, TranscriptWord } from "./transcript-words";

const word = (text: string, startMs: number): TranscriptWord => ({
  endMs: startMs + 400,
  probability: 1,
  startMs,
  text,
});

// "One two." then a 2.6 s pause, then "Three four five", ending too close
// to the end of the recording for a pause to show after it.
const words = [
  word("One", 0),
  word("two.", 500),
  word("Three", 3_500),
  word("four", 4_000),
  word("five", 4_500),
];
const DURATION_MS = 5_000;

const heardWith = (removed: { endMs: number; startMs: number }[] = []) =>
  transcriptUnits(words, [], { durationMs: DURATION_MS, removed });

const texts = (units: readonly { kind: string; text: string }[]) =>
  units.map((unit) => (unit.kind === "pause" ? "_" : unit.text)).join(" ");

// 3.5-5 s plays first, then 0-3.5 s.
const swapped = [
  { endMs: 5_000, startMs: 3_500 },
  { endMs: 3_500, startMs: 0 },
];

describe("transcriptInPlaybackOrder", () => {
  it("reads the words in the order the timeline plays them", () => {
    const order = transcriptInPlaybackOrder(heardWith(), swapped);
    expect(texts(order.units)).toEqual("Three four five One two. _");
    expect(order.shownIndex[order.heardIndex[2]]).toBe(2);
  });

  it("keeps a cut word where it would be restored, after the footage before it", () => {
    // "four" cut away, and 4.5-5 s played first.
    const order = transcriptInPlaybackOrder(
      heardWith([{ endMs: 4_500, startMs: 4_000 }]),
      [
        { endMs: 5_000, startMs: 4_500 },
        { endMs: 4_000, startMs: 0 },
      ],
    );
    expect(texts(order.units)).toEqual("five One two. _ Three four");
  });

  it("keeps the paragraph a pause ended where it was heard", () => {
    // The pause after "two." played after "Three" instead.
    const order = transcriptInPlaybackOrder(heardWith(), [
      { endMs: 900, startMs: 0 },
      { endMs: 3_900, startMs: 3_500 },
      { endMs: 3_500, startMs: 900 },
      { endMs: 5_000, startMs: 3_900 },
    ]);
    expect(texts(order.units)).toEqual("One two. Three _ four five");
    const [first] = transcriptParagraphs(order.units);
    expect(texts(order.units.slice(first.start, first.end))).toEqual(
      "One two. Three _",
    );
  });
});

describe("heardRuns", () => {
  it("parts what is shown together into the runs it was heard in", () => {
    const order = transcriptInPlaybackOrder(heardWith(), swapped);
    // "five One two.": the end of what plays first and the start of the
    // rest.
    const span = { first: 2, last: 4 };
    expect(heardRuns(order, span)).toEqual([
      { first: 0, last: 1 },
      { first: 5, last: 5 },
    ]);
    expect(isHeardInOrder(order, span)).toBe(false);
    expect(isHeardInOrder(order, { first: 0, last: 2 })).toBe(true);
  });
});

describe("movedFootage", () => {
  const order = transcriptInPlaybackOrder(heardWith(), []);
  const at = (text: string) =>
    order.units.findIndex((unit) => unit.text === text);

  it("takes the space after the last word with it when no pause shows there", () => {
    expect(
      movedFootage(order, { first: at("Three"), last: at("four") }, at("One")),
    ).toEqual({ beforeMs: 0, endMs: 4_500, startMs: 3_500 });
  });

  it("leaves a pause that shows after the words where it is", () => {
    expect(
      movedFootage(
        order,
        { first: at("One"), last: at("two.") },
        order.units.length,
      ),
    ).toEqual({ beforeMs: null, endMs: 900, startMs: 0 });
  });

  it("moves nothing for words that are all cut", () => {
    const cut = transcriptInPlaybackOrder(
      heardWith([{ endMs: 900, startMs: 0 }]),
      [],
    );
    expect(movedFootage(cut, { first: 0, last: 1 }, 4)).toBeNull();
  });
});
