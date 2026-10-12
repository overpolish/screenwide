// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { findInTranscript } from "./transcript-search";
import { TranscriptUnit } from "./transcript-units";

const unit = (text: string, startMs: number): TranscriptUnit => ({
  corrected: false,
  endMs: startMs + 400,
  heard: text,
  kind: "word",
  removed: false,
  startMs,
  text,
  unsure: false,
});

const units = [
  unit("Open", 0),
  unit("the", 500),
  unit("Café,", 1_000),
  unit("then", 1_500),
  unit("open", 2_000),
  unit("the", 2_500),
  unit("door.", 3_000),
];

describe("findInTranscript", () => {
  it("finds every match regardless of case, accents and punctuation", () => {
    expect(findInTranscript(units, "cafe")).toEqual([
      { endOffset: 4, first: 2, last: 2, startOffset: 0 },
    ]);
    expect(findInTranscript(units, "OPEN")).toEqual([
      { endOffset: 4, first: 0, last: 0, startOffset: 0 },
      { endOffset: 4, first: 4, last: 4, startOffset: 0 },
    ]);
  });
  it("runs a phrase across words, taking every word it touches", () => {
    expect(findInTranscript(units, "e caf")).toEqual([
      { endOffset: 3, first: 1, last: 2, startOffset: 2 },
    ]);
    expect(findInTranscript(units, "the door")).toEqual([
      { endOffset: 4, first: 5, last: 6, startOffset: 0 },
    ]);
  });

  it("maps a match beginning and ending inside a word to its own characters", () => {
    expect(findInTranscript([unit("Worldwide", 0)], "wide")).toEqual([
      { endOffset: 9, first: 0, last: 0, startOffset: 5 },
    ]);
    expect(findInTranscript([unit("Worldwide", 0)], "world")).toEqual([
      { endOffset: 5, first: 0, last: 0, startOffset: 0 },
    ]);
  });

  it("maps an accented character to its single original index", () => {
    expect(findInTranscript([unit("café", 0)], "afe")).toEqual([
      { endOffset: 4, first: 0, last: 0, startOffset: 1 },
    ]);
  });

  it("searches the corrected text rather than what was heard", () => {
    const corrected = [{ ...unit("doors", 0), heard: "door" }];
    expect(findInTranscript(corrected, "doors")).toEqual([
      { endOffset: 5, first: 0, last: 0, startOffset: 0 },
    ]);
  });

  it("finds nothing for a query of spaces or punctuation alone", () => {
    expect(findInTranscript(units, " ,. ")).toEqual([]);
  });
});
