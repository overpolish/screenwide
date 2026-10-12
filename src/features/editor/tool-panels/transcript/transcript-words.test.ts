// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  isWordRemoved,
  nearestKeptWord,
  transcriptParagraphs,
  TranscriptWord,
  wordAt,
  wordRuns,
  wordRunSpans,
} from "./transcript-words";

const word = (
  text: string,
  startMs: number,
  endMs: number,
): TranscriptWord => ({
  endMs,
  probability: 1,
  startMs,
  text,
});

const words = [
  word("One", 0, 400),
  word("two", 500, 900),
  word("three.", 1_000, 1_400),
  word("Four", 3_000, 3_400),
  word("five", 3_500, 3_900),
  word("six", 8_000, 8_400),
];

// Paragraphs take the units the panel shows: words, with a pause unit
// wherever the transcript put one - the two short, sub-second gaps in
// `words` never get one.
const paragraphWord = (w: TranscriptWord) => ({
  ...w,
  heard: w.text,
  kind: "word" as const,
});
const pause = (startMs: number, endMs: number) => ({
  endMs,
  heard: "",
  kind: "pause" as const,
  startMs,
});

describe("transcriptParagraphs", () => {
  it("breaks after a pause that ends a sentence, or after any long pause", () => {
    const items = [
      paragraphWord(words[0]),
      paragraphWord(words[1]),
      paragraphWord(words[2]),
      pause(1_400, 3_000),
      paragraphWord(words[3]),
      paragraphWord(words[4]),
      pause(3_900, 8_000),
      paragraphWord(words[5]),
    ];
    expect(transcriptParagraphs(items)).toEqual([
      { end: 4, start: 0 },
      { end: 7, start: 4 },
      { end: 8, start: 7 },
    ]);
  });

  it("keeps a sentence together where no pause unit sits between the words", () => {
    expect(
      transcriptParagraphs([
        paragraphWord(word("Wait.", 0, 400)),
        paragraphWord(word("Then", 1_000, 1_400)),
      ]),
    ).toEqual([{ end: 2, start: 0 }]);
  });
});

describe("isWordRemoved", () => {
  it("strikes a word whose middle a cut covers, not one it only clips", () => {
    const removed = [{ endMs: 750, startMs: 300 }];
    expect(isWordRemoved(words[0], removed)).toBe(false);
    expect(isWordRemoved(words[1], removed)).toBe(true);
    expect(isWordRemoved(words[2], removed)).toBe(false);
  });
});

describe("wordAt", () => {
  it("finds the word being said and nothing between words", () => {
    expect(wordAt(words, 600)).toBe(1);
    expect(wordAt(words, 950)).toBe(-1);
    expect(wordAt(words, 900)).toBe(-1);
  });
});

describe("wordRuns", () => {
  const flags = [false, true, false, false, true, false];

  it("splits a selection into the runs it keeps, around struck words", () => {
    expect(
      wordRuns(words, flags, { first: 0, last: 3, removed: false }),
    ).toEqual([
      { endMs: 400, startMs: 0 },
      { endMs: 3_400, startMs: 1_000 },
    ]);
  });

  it("finds the struck runs a selection covers for restoring", () => {
    expect(
      wordRuns(words, flags, { first: 1, last: 5, removed: true }),
    ).toEqual([
      { endMs: 900, startMs: 500 },
      { endMs: 3_900, startMs: 3_500 },
    ]);
  });
});

describe("wordRunSpans", () => {
  const flags = [false, true, false, false, true, false];

  it("gives the runs as indices, for a cut to pad by neighbour", () => {
    expect(wordRunSpans(flags, { first: 0, last: 3, removed: false })).toEqual([
      { first: 0, last: 0 },
      { first: 2, last: 3 },
    ]);
  });
});

describe("nearestKeptWord", () => {
  const flags = [false, true, true, false];

  it("steps over struck words in the direction of the delete", () => {
    expect(nearestKeptWord(flags, 2, -1)).toBe(0);
    expect(nearestKeptWord(flags, 1, 1)).toBe(3);
    expect(nearestKeptWord(flags, -1, -1)).toBe(-1);
    expect(nearestKeptWord(flags, 4, 1)).toBe(-1);
  });
});
