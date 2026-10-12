// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { findFillerWords } from "./transcript-fillers";
import { TranscriptUnit } from "./transcript-units";

const unit = (
  text: string,
  startMs: number,
  overrides: Partial<TranscriptUnit> = {},
): TranscriptUnit => ({
  corrected: false,
  endMs: startMs + 300,
  heard: text,
  kind: "word",
  removed: false,
  startMs,
  text,
  unsure: false,
  ...overrides,
});

describe("findFillerWords", () => {
  it("finds a hesitation sound however it is written out", () => {
    const units = [unit("Well,", 0), unit("Ummm,", 500), unit("uh", 1_000)];
    expect(findFillerWords(units, "en")).toEqual([
      { folded: "um", span: { first: 1, last: 1 } },
      { folded: "uh", span: { first: 2, last: 2 } },
    ]);
  });

  it("leaves phrases that are often meant alone", () => {
    const units = [unit("You", 0), unit("know,", 350), unit("I", 1_000)];
    expect(findFillerWords([...units, unit("mean", 1_350)], "en")).toEqual([]);
  });

  it("finds the transcript language's own sounds as well as English's", () => {
    const units = [unit("Äh,", 0), unit("um", 500)];
    expect(findFillerWords(units, "de").map(({ folded }) => folded)).toEqual([
      "äh",
      "um",
    ]);
    expect(findFillerWords(units, "en").map(({ folded }) => folded)).toEqual([
      "um",
    ]);
  });

  it("marks the first of a word said again at once, not one said again later", () => {
    const quick = [unit("the", 0), unit("the", 400), unit("tool", 800)];
    expect(findFillerWords(quick, "en")).toEqual([
      { folded: "the", span: { first: 0, last: 0 } },
    ]);
    const later = [unit("the", 0), unit("the", 1_500)];
    expect(findFillerWords(later, "en")).toEqual([]);
  });

  it("never marks a word already cut or a pause", () => {
    const units = [
      unit("um", 0, { removed: true }),
      unit("", 400, { kind: "pause" }),
      unit("uh", 1_400),
    ];
    expect(findFillerWords(units, "en")).toEqual([
      { folded: "uh", span: { first: 2, last: 2 } },
    ]);
  });

  it("reads a corrected word as it is shown, not as it was heard", () => {
    const units = [
      unit("um", 0, { corrected: true, heard: "well" }),
      unit("well", 500, { corrected: true, heard: "um" }),
    ];
    expect(findFillerWords(units, "en")).toEqual([
      { folded: "um", span: { first: 0, last: 0 } },
    ]);
  });
});
