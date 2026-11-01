// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { captionPart, SLIVER_S, splitAt } from "./strip-split";

const own = <T>(item: T) => item;

describe("captionPart", () => {
  const line = {
    end: 7.136,
    start: 6.592,
    text: "Howdy there",
    words: [
      { at: 6.592, word: "Howdy" },
      { at: 6.9, word: "there" },
    ],
  };

  it("keeps a line's first word with it at a jump a rounding after its start", () => {
    // A moved stretch's start, summed in floating point.
    const parts = splitAt([line], [6.5920000000000005], {
      part: captionPart,
      spanOf: own,
    }).filter(({ end, start }) => end - start >= SLIVER_S);
    expect(parts.map(({ text }) => text)).toEqual(["Howdy there"]);
  });

  it("gives each word to the part it is said in", () => {
    const parts = splitAt([line], [6.8], { part: captionPart, spanOf: own });
    expect(parts.map(({ text }) => text)).toEqual(["Howdy", "there"]);
  });
});
