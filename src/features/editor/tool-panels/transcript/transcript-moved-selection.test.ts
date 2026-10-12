// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { movedSelection } from "./transcript-moved-selection";
import { TranscriptUnit } from "./transcript-units";

const unit = (text: string): TranscriptUnit => ({
  corrected: false,
  endMs: 0,
  heard: text,
  kind: text === "" ? "pause" : "word",
  removed: false,
  startMs: 0,
  text,
  unsure: false,
});
const units = (text: string) =>
  text.split(" ").map((word) => unit(word === "_" ? "" : word));

describe("movedSelection", () => {
  it("finds the carried words where they landed", () => {
    expect(movedSelection(units("a b c d"), units("b c"), 0)).toEqual({
      anchor: 1,
      focus: 2,
    });
  });

  it("looks only for the words when a pause at an end joined another", () => {
    // The pause carried before "b" merged with the one already there.
    expect(movedSelection(units("a _ b c d"), units("_ b c"), 1)).toEqual({
      anchor: 2,
      focus: 3,
    });
  });

  it("takes the occurrence nearest where the words were dropped", () => {
    expect(movedSelection(units("the x the y"), units("the"), 3)).toEqual({
      anchor: 2,
      focus: 2,
    });
  });

  it("finds nothing for a carried run of pauses or words that are gone", () => {
    expect(movedSelection(units("a b"), units("_"), 0)).toBeNull();
    expect(movedSelection(units("a b"), units("z"), 0)).toBeNull();
  });
});
