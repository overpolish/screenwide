// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { joinedCutRange } from "./transcript-cut-join";

const word = (startMs: number, endMs: number) => ({
  endMs,
  kind: "word" as const,
  startMs,
});
const pause = (startMs: number, endMs: number) => ({
  endMs,
  kind: "pause" as const,
  startMs,
});

describe("joinedCutRange", () => {
  it("cuts a word inside a sentence and nothing more", () => {
    const units = [word(0, 300), word(300, 700), word(700, 1_000)];
    expect(joinedCutRange(units, { first: 1, last: 1 }, 2_000)).toEqual({
      endMs: 700,
      startMs: 300,
    });
  });

  it("keeps the break after a sentence whose last word goes", () => {
    const units = [
      word(0, 300),
      word(300, 700),
      pause(700, 2_900),
      word(2_900, 3_300),
    ];
    expect(joinedCutRange(units, { first: 1, last: 1 }, 4_000)).toEqual({
      endMs: 700,
      startMs: 300,
    });
  });

  it("keeps one pause around a filler, split across the join", () => {
    const units = [word(0, 300), word(500, 700), word(1_000, 1_300)];
    expect(joinedCutRange(units, { first: 1, last: 1 }, 2_000)).toEqual({
      endMs: 850,
      startMs: 450,
    });
  });

  it("leaves one sentence break where a sentence between two goes", () => {
    const units = [
      word(0, 1_000),
      pause(1_000, 2_000),
      word(2_000, 3_000),
      pause(3_000, 4_200),
      word(4_200, 5_000),
    ];
    // The chosen pause before the sentence goes with it, and the join keeps
    // the longer of the breaks, 1 200 ms.
    expect(joinedCutRange(units, { first: 1, last: 2 }, 6_000)).toEqual({
      endMs: 3_600,
      startMs: 1_600,
    });
  });

  it("lets the recording's start and end stand in for a missing neighbour", () => {
    const units = [word(800, 1_200), word(1_500, 2_000)];
    expect(joinedCutRange(units, { first: 0, last: 0 }, 3_000)).toEqual({
      endMs: 1_200,
      startMs: 500,
    });
    expect(joinedCutRange(units, { first: 1, last: 1 }, 3_000)).toEqual({
      endMs: 2_300,
      startMs: 1_500,
    });
  });

  it("keeps a beat of a cut pause beside the words", () => {
    const units = [word(0, 500), pause(500, 2_500), word(2_500, 3_000)];
    expect(joinedCutRange(units, { first: 1, last: 1 }, 4_000)).toEqual({
      endMs: 2_400,
      startMs: 600,
    });
    const opening = [pause(0, 1_800), word(1_800, 2_200)];
    expect(joinedCutRange(opening, { first: 0, last: 0 }, 3_000)).toEqual({
      endMs: 1_600,
      startMs: 0,
    });
  });
});
