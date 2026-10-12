// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { findProfanity } from "./transcript-profanity";
import { loadProfanity } from "./transcript-profanity-lists";
import { TranscriptUnit } from "./transcript-units";

const unit = (
  text: string,
  overrides: Partial<TranscriptUnit> = {},
): TranscriptUnit => ({
  corrected: false,
  endMs: 0,
  heard: text,
  kind: "word",
  removed: false,
  startMs: 0,
  text,
  unsure: false,
  ...overrides,
});

const lexicon = {
  phrases: new Map([["son", [["son", "of", "a", "bitch"]]]]),
  unspaced: ["笨蛋"],
  words: new Set(["damn", "bitch"]),
};

describe("findProfanity", () => {
  it("finds a word however it is cased or punctuated", () => {
    expect(
      findProfanity([unit("Oh"), unit("DAMN!")], lexicon, new Set()),
    ).toEqual([{ folded: "damn", span: { first: 1, last: 1 } }]);
  });

  it("takes the longest phrase at a word as one finding", () => {
    const units = ["You", "son", "of", "a", "bitch"].map((text) => unit(text));
    expect(findProfanity(units, lexicon, new Set())).toEqual([
      { folded: "son of a bitch", span: { first: 1, last: 4 } },
    ]);
  });

  it("finds an unspaced entry across the words it runs over", () => {
    const units = [unit("你"), unit("笨"), unit("蛋")];
    expect(findProfanity(units, lexicon, new Set())).toEqual([
      { folded: "笨蛋", span: { first: 1, last: 2 } },
    ]);
  });

  it("leaves out words marked not profanity and words already cut", () => {
    const units = [unit("damn"), unit("bitch", { removed: true })];
    expect(findProfanity(units, lexicon, new Set(["damn"]))).toEqual([]);
  });
});

describe("loadProfanity", () => {
  it("reads English for every language, with the language's own list", async () => {
    const german = await loadProfanity("de");
    expect(german.words.has("damn")).toBe(true);
    expect(german.words.has("arschloch")).toBe(true);
    const english = await loadProfanity("en");
    expect(english.words.has("arschloch")).toBe(false);
  });

  it("finds a list for every language that has one, and reads none of its notes", async () => {
    const english = await loadProfanity("en");
    const size = (lexicon: typeof english) =>
      lexicon.words.size + lexicon.phrases.size + lexicon.unspaced.length;
    for (const language of ["ar", "ja", "ms", "nn", "th", "yue", "zh"]) {
      const lexicon = await loadProfanity(language);
      expect(size(lexicon), language).toBeGreaterThan(size(english));
      expect(lexicon.words.has("spdxlicenseidentifier")).toBe(false);
    }
    // A language without a list marks English swearing alone.
    expect(size(await loadProfanity("sw"))).toBe(size(english));
  });
});
