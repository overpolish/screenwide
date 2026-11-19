// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { describe, expect, it } from "vitest";

import { markLines } from "./mark-lines";

describe("the lines cues' marks are drawn on", () => {
  it("draws each beside its word's line when no two share one", () => {
    expect(markLines([0, 2, 3], 5)).toEqual([0, 2, 3]);
  });

  it("moves a later cue sharing a line down to the next line", () => {
    expect(markLines([0, 0], 3)).toEqual([0, 1]);
  });

  it("passes over a line another cue's word is on", () => {
    expect(markLines([0, 0, 1], 4)).toEqual([0, 2, 1]);
  });

  it("passes over a line a moved mark has taken", () => {
    expect(markLines([0, 0, 0], 4)).toEqual([0, 1, 2]);
  });

  it("goes up to the nearest free line where none is free below", () => {
    expect(markLines([0, 3, 3], 4)).toEqual([0, 3, 2]);
  });

  it("stays on its own line when every line is taken", () => {
    expect(markLines([0, 1, 1], 2)).toEqual([0, 1, 1]);
  });
});
