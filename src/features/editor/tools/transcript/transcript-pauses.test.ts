// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { pausesCutBy, transcriptPauses } from "./transcript-pauses";
import { TranscriptWord } from "./transcript-words";

const word = (startMs: number, endMs: number): TranscriptWord => ({
  endMs,
  probability: 1,
  startMs,
  text: "word",
});

describe("transcriptPauses", () => {
  it("finds gaps between words long enough to hear", () => {
    const words = [word(0, 400), word(500, 900), word(2_000, 2_400)];
    expect(transcriptPauses(words)).toEqual([{ endMs: 2_000, startMs: 900 }]);
  });

  it("starts showing a gap at 300 ms", () => {
    expect(transcriptPauses([word(0, 400), word(699, 900)])).toEqual([]);
    expect(transcriptPauses([word(0, 400), word(700, 900)])).toEqual([
      { endMs: 700, startMs: 400 },
    ]);
  });

  it("honours a given threshold", () => {
    const words = [word(0, 400), word(900, 1_300)];
    expect(transcriptPauses(words, { minMs: 400 })).toEqual([
      { endMs: 900, startMs: 400 },
    ]);
    expect(transcriptPauses(words, { minMs: 600 })).toEqual([]);
  });

  it("shows the silence before the first word and after the last", () => {
    const words = [word(1_500, 1_900), word(2_000, 2_400)];
    expect(transcriptPauses(words, { durationMs: 5_000 })).toEqual([
      { endMs: 1_500, startMs: 0 },
      { endMs: 5_000, startMs: 2_400 },
    ]);
    expect(transcriptPauses(words, { durationMs: 2_600 })).toEqual([
      { endMs: 1_500, startMs: 0 },
    ]);
  });

  it("treats a recording with no words as one silence", () => {
    expect(transcriptPauses([], { durationMs: 4_000 })).toEqual([
      { endMs: 4_000, startMs: 0 },
    ]);
  });
});

describe("pausesCutBy", () => {
  const word = (startMs: number, endMs: number) => ({
    endMs,
    kind: "word" as const,
    removed: false,
    startMs,
  });
  const pause = (startMs: number, endMs: number, removed = false) => ({
    endMs,
    kind: "pause" as const,
    removed,
    startMs,
  });

  it("marks only kept pauses a planned cut reaches", () => {
    const units = [
      word(0, 500),
      pause(500, 2_000),
      word(2_000, 2_400),
      pause(2_400, 2_800),
      word(2_800, 3_200),
      pause(3_200, 5_000, true),
    ];
    const cuts = [
      { endMs: 1_850, startMs: 750 },
      { endMs: 4_850, startMs: 3_450 },
    ];
    expect([...pausesCutBy(units, cuts)]).toEqual([1]);
  });
});
