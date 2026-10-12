// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  correctionsWith,
  isCorrectionApplied,
  transcriptUnits,
} from "./transcript-units";
import { TranscriptWord } from "./transcript-words";

const word = (
  text: string,
  startMs: number,
  probability = 1,
): TranscriptWord => ({ endMs: startMs + 400, probability, startMs, text });

const words = [
  word("I", 0),
  word("iced", 500, 0.3),
  word("cream", 1_000),
  word("alot", 1_500),
];

describe("transcriptUnits", () => {
  it("joins the words a correction covers under its text", () => {
    const units = transcriptUnits(
      words,
      [{ endMs: 1_400, startMs: 500, text: "ice cream" }],
      { removed: [] },
    );
    expect(units.map((unit) => unit.text)).toEqual(["I", "ice cream", "alot"]);
    expect(units[1]).toMatchObject({
      corrected: true,
      endMs: 1_400,
      heard: "iced cream",
      startMs: 500,
      unsure: false,
    });
  });

  it("keeps an unsure word marked until it is corrected", () => {
    expect(transcriptUnits(words, [], { removed: [] })[1].unsure).toBe(true);
  });

  it("does not count spacing alone as a correction", () => {
    const units = transcriptUnits(
      words,
      [{ endMs: 1_900, startMs: 1_500, text: " alot  " }],
      { removed: [] },
    );
    expect(units[3].corrected).toBe(false);
  });

  it("shows a pause unit in a gap long enough to hear, never corrected", () => {
    const paused = [word("Hi", 0), word("there", 2_000)];
    const units = transcriptUnits(paused, [], { removed: [] });
    expect(units).toHaveLength(3);
    expect(units[1]).toMatchObject({
      corrected: false,
      endMs: 2_000,
      kind: "pause",
      startMs: 400,
      text: "",
    });
    expect(units[0].kind).toBe("word");
    expect(units[2].kind).toBe("word");
  });
});

describe("correctionsWith", () => {
  const phrase = { endMs: 1_400, startMs: 500, text: "ice cream" };

  it("replaces every correction the new one overlaps", () => {
    const corrected = correctionsWith(
      words,
      [
        { endMs: 900, startMs: 500, text: "ice" },
        { endMs: 1_900, startMs: 1_500, text: "a lot" },
      ],
      phrase,
    );
    expect(corrected).toEqual([
      phrase,
      { endMs: 1_900, startMs: 1_500, text: "a lot" },
    ]);
  });

  it("takes a correction away when its text is what was heard", () => {
    expect(
      correctionsWith(words, [phrase], { ...phrase, text: "iced  cream" }),
    ).toEqual([]);
  });

  it("knows a change the editor has already published", () => {
    expect(
      isCorrectionApplied(words, [phrase], { ...phrase, text: "ice cream " }),
    ).toBe(true);
    expect(isCorrectionApplied(words, [], phrase)).toBe(false);
  });
});
